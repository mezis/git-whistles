#!/usr/bin/env ruby
# encoding: UTF-8
#
# git-jira-branch
# 
#   Suggest a branch name from the given JIRA issue ID
# 
#   Assumes the branches are named as:
#   <team>/<branch-title>-<story-id>
# 
require 'rubygems'
require 'optparse'
require 'jira'
require 'readline'
require 'term/ansicolor'
require 'git-whistles/app'

class App < Git::Whistles::App

  def initialize
    super
  end

  def main(args)
    super
    parse_args!(args)

    if args.count < 1
      show_and_exit usage
    end

    story_id = args[0]

    # get PT info
    issue = get_jira_info(story_id)

    branch_name_suggested = "#{story_id.upcase}-#{issue.fields['summary'].downcase}"
    branch_name_suggested.gsub!(/[^\w\d\/]/, '-').gsub!(/-+/, '-')

    puts 'The suggested branch name is: ' << Term::ANSIColor.yellow(branch_name_suggested)
    puts '    Press ENTER if you agree'
    puts ' or Write any other name and press ENTER'
    puts ' or Press CTRL-D to cancel'

    branch_name = Readline.readline '  > ', false

    if branch_name.nil?
      log.warn "\nCancelled by user"
      exit 2
    end

    puts 'Would you like to transition the issue?'
    issue.transitions.all.each do |t|
      puts "#{t.id} - #{t.name}"
    end
    puts 'Type the number or enter to continue'
    transition = Readline.readline '  > ', false

    branch_name = branch_name.empty? ? branch_name_suggested : branch_name

    # create branch
    `cd #{ENV['PWD']} && git checkout -b #{branch_name}`

    # comment and start on PT
    # issue.notes.create :text => "Created branch #{branch_name}"
    # issue.update :current_state => 'started'

    if transition
      # Seems we need to do this in two updates.  JIRA complained if we tried to transition and update fields.
      attrs = {
        update: {
          comment: [{
                      add: {
                        body: "Created branch #{branch_name}"
                      }
                    }],
          assignee: [{
                       set: {
                         name: @username
                       }
                     }]
        }
      }
      issue.save!(attrs)
      attrs = {
        transition: {
          id: transition
        }
      }
      issue.client.post("#{issue.url}/transitions", attrs.to_json)
    else
      attrs = {
        update: {
          assignee: [{
                       set: {
                         name: @username
                       }
                     }],
          comment: [{
                      add: {
                        body: "Created branch #{branch_name}"
                      }
                    }]
        }
      }
      issue.save!(attrs)
    end

    puts Term::ANSIColor.green('Created branch and started JIRA issue')
  end

  private

  def usage
    'Usage: git jira-branch JIRA_ID'
  end

  def show_and_exit(message)
    puts message
    exit 1
  end

  def option_parser
    @option_parser ||= OptionParser.new do |op|
      op.banner = usage

      op.on_tail('-h', '--help', 'Show this message') do
        show_and_exit op
      end
    end
  end

  def get_jira_info(story_id)
    @username = `git config jira.username`.strip
    if @username.empty?
      puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a story ID,
        but I don't know your JIRA username!
        Please set it with:
        $ git config [--global] jira.username <token>
      }
      die "Aborting."
    end

    password = `git config jira.password`.strip
    if password.empty?
      puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a story ID,
        but I don't know your JIRA password!
        Please set it with:
        $ git config [--global] jira.password <password>
      }
      die "Aborting."
    end

    site = `git config jira.site`.strip
    if site.empty?
      puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a story ID,
        but I don't know your JIRA site!
        Please set it with:
        $ git config [--global] jira.site <https://mydomain.atlassian.net>
      }
      die "Aborting."
    end
    log.info "Finding your project and story¬"

    options = {
      username: @username,
      password: password,
      site: site,
      context_path: '',
      auth_type: :basic,
      read_timeout: 120
    }

    client = JIRA::Client.new(options)
    issue = client.Issue.find(story_id)

    log.info '.'

    log.info "Found story #{story_id} in '#{issue.fields['project']['name']}'"

    issue
  rescue => e
    log.info e.message

    log.warn "Apologies... I could not find story #{story_id}."
    exit 1
  end


end

############################################################################

App.run!
