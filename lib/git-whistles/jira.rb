module Git
  module Whistles
    module Jira
      def get_config
        username = `git config jira.username`.strip
        if username.empty?
          puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a issue ID,
        but I don't know your JIRA username!
        Please set it with:
        $ git config [--global] jira.username <username>
      }
          die "Aborting."
        end

        password = `git config jira.password`.strip
        if password.empty?
          puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a issue ID,
        but I don't know your JIRA password!
        Please set it with:
        $ git config [--global] jira.password <password>
      }
          die "Aborting."
        end

        site = `git config jira.site`.strip
        if site.empty?
          puts Term::ANSIColor.yellow %Q{
        Your branch appears to have a issue ID,
        but I don't know your JIRA site!
        Please set it with:
        $ git config [--global] jira.site <https://mydomain.atlassian.net>
      }
          die "Aborting."
        end

        { username: username, password: password, site: site }
      end

      def get_client(opts = get_config)
        options = {
          context_path: '',
          auth_type: :basic,
          read_timeout: 120
        }

        options.merge!(opts)

        JIRA::Client.new(options)
      end
    end
  end
end
