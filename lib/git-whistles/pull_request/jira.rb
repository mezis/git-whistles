require 'jira-ruby'
require 'git-whistles/jira'
require_relative 'bare'

module Git
  module Whistles
    module PullRequest
      class Jira < Bare
        private

        def pr_command
          'jira-pr'
        end

        def tracker_related_params(team)
          params = {}

          if options.from =~ %r{^(\w+-\w+)/.*}
            team, issue_id = $1.capitalize.split('-')
          else
            issue_id = team = nil
          end
          issue_id = "#{team}-#{issue_id}".upcase if issue_id =~ /(\d+)$/

          log.info 'Finding your Jira Issue¬'

          client = Git::Whistles::Jira.new.get_client rescue die('Aborting.')
          issue = client.Issue.find(issue_id)
          log.info '.'

          if issue.nil?
            log.warn "Apologies… I could not find issue #{issue_id}."
            die
          end

          log.info "Found story #{issue_id} in '#{issue.fields['project']['name']}'"

          title       = "#{issue_id}: #{issue.summary}"
          headline    = "Jira story [##{issue_id}](#{client.options[:site]}/browse/#{issue_id}) in project *#{issue.project.name.strip}*:"

          description = safe_description(issue.description)
          params[:subject] = issue.summary
          params[:'pull_request[title]'] = title

          if (headline.length + description.length) > SAFE_QUERY_STRING_SIZE
            log.warn 'Oops looks like your story body exceeds maximum allowed caracters to send a github request'
            log.warn 'Please copy the info below to your pull request body:'
            puts
            puts headline
            puts
            puts
            puts description
            puts
            puts
            puts 'Press any key to continue…'
            gets
            params.merge! :"pull_request[body]" => 'Please check your command line for the story body'
          else
            body = "TODO: describe your changes\n\n===\n\n#{headline}\n\n#{description}"
            params[:'pull_request[body]'] = body
          end
          params
        end

        def safe_description(description)
          return '' unless description

          description.split("\n").map do |line|
            (1..6).each { |i| line.gsub!(/(h#{i}.)/, '#' * i) }
            line.gsub!(/({{)|(}})/, '`')
            "> #{line}"
          end.join("\n")
        end
      end
    end
  end
end
