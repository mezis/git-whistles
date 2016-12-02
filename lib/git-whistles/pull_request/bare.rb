module Git
  module Whistles
    module PullRequest
      class Bare < Git::Whistles::App
        BROWSERS               = %w(xdg-open open firefox iceweasel)
        SAFE_QUERY_STRING_SIZE = 8000

        def initialize
          super
        end

        def main(args)
          super

          parse_args!(args)

          if args.count > 0
            die 'Too many arguments', :usage => true
          end

          if options.from == options.to
            die "You cannot issue a pull request to the same branch (#{options.from})."
          end

          query = {}

          # guess team name
          if options.from =~ %r{^(\w+)/.*}
            team = $1.capitalize
          else
            team = nil
          end

          # guess title.
          title = options.from.split('/').last.split(/[_-]/).delete_if { |word| word =~ /^\d+$/ }.join(' ').capitalize
          query[:"pull_request[title]"] = team ? "#{team}: #{title}" : title

          query.merge!(tracker_related_params(team))

          query_string = query.map { |key,value|
            "#{CGI.escape key.to_s}=#{CGI.escape value}"
          }.join('&')
          url = "https://github.com/#{repo}/compare/#{options.to}...#{options.from}?#{query_string}"

          puts "Preparing a pull request for branch #{options.from}"

          unless launch_browser(url)
            log.warn "Sorry, I don't know how to launch a web browser on your system. You can open it yourself and paste this URL:\n#{url}"
          end
        end

        private

        def pr_command # to be overridden by subclass
          'pr'
        end

        def tracker_related_params(team)
          {} # to be overridden by tracker-specific subclass
        end

        def origin_url
          @origin_url ||= begin
            run!("git config --get remote.#{options.remote}.url").strip.tap do |url|
              url =~ /github\.com/ or die 'origin does not have a Github URL !'
            end
          end
        end

        def repo
          @repo ||= origin_url.sub(/.*github\.com[\/:]/, '').sub(/\.git$/, '')
        end

        def launch_browser(url)
          BROWSERS.each do |command|
            next if run("which #{command}").strip.empty?
            system(command, url) and return true
          end
          false
        end

        def defaults
          {
            from:   run!('git symbolic-ref HEAD').strip.gsub(%r(^refs/heads/), ''),
            to:     'master',
            remote: 'origin'
          }
        end

        def option_parser
          @option_parser ||= OptionParser.new do |op|
            op.banner = "Usage: git #{pr_command} [options]"

            op.on("-f", "--from YOUR_BRANCH", "Branch to issue pull request for [head]") do |v|
              options.from = v
            end

            op.on("-to", "--to UPSTREAM_BRANCH", "Branch into which you want your code merged [master]") do |v|
              options.to = v
            end

            op.on("-r", "--remote NAME", "The remote you're sending this to [origin]") do |v|
              options.to = v
            end
          end
        end
      end
    end
  end
end
