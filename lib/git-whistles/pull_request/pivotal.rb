require 'tracker_api'
require_relative 'bare'

module Git
  module Whistles
    module PullRequest
      class Pivotal < Bare
        private

        def pr_command
          'pivotal-pr'
        end

        def tracker_related_params(team)
          params = {}
          story_id = $1.to_i if options.from =~ /(\d+)$/

          token = `git config pivotal-tracker.token`.strip
          if token.empty?
            puts Term::ANSIColor.yellow %Q{
              Your branch appears to have a story ID,
              but I don't know your Pivotal Tracker token!
              Please set it with:
              $ git config [--global] pivotal-tracker.token <token>
            }
            die 'Aborting.'
          end

          log.info 'Finding your project and story¬'

          client = TrackerApi::Client.new(token: token)
          begin
            story, project = client.projects.find do |project|
              log.info '.¬'
              story = project.story(story_id) and break story, project
            end
            log.info '.'
          rescue StandardError => e
            raise unless e.kind_of?(TrackerApi::Errors::ClientError) && e.response[:status] == 403

            log.info '.'
            die "Your token is not authorized by Pivotal Tracker! Please make sure you have the correct one"
          end

          if story.nil?
            log.warn "Apologies… I could not find story #{story_id}."
            die
          end

          log.info "Found story #{story_id} in '#{project.name}'"

          title       = "#{project.name}: #{story.name} [##{story.id}]"
          headline    = "Pivotal tracker story [##{story_id}](#{story.url}) in project *#{project.name.strip}*:"
          description = story.description.split("\n").map { |line| "> #{line}" }.join("\n")
          params[:subject] = story.name
          params[:'pull_request[title]'] = title

          if (headline.length + description.length) > SAFE_QUERY_STRING_SIZE
            log.warn "Oops looks like your story body exceeds maximum allowed caracters to send a github request"
            log.warn "Please copy the info below to your pull request body:"
            puts
            puts headline
            puts
            puts
            puts description
            puts
            puts
            puts 'Press any key to continue…'
            gets
            params.merge! :"pull_request[body]" => "Please check your command line for the story body"
          else
            body = "TODO: describe your changes\n\n===\n\n#{headline}\n\n#{description}"
            params[:'pull_request[body]'] = body
          end
          params
        end
      end
    end
  end
end
