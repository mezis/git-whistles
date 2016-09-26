require 'youtrack'
require_relative 'ticket'

module Git::Whistles
  module Youtrack
    class Api
      def get_ticket(id)
        Ticket.build_from_remote(issues.find(id))
      end

      def issues
        @issues ||= client.issues
      end

      def client
        @client ||= ::Youtrack::Client.new do |c|
          c.url = url
          c.login = username
          c.password = password
        end

        @client.connect! unless @client.connected?
        @client
      end

      def username
        username = `git config youtrack.username`.strip

        if username.empty?
          puts Term::ANSIColor.yellow %Q{
            Can't find Youtrack username!
            Please set it with:
            $ git config [--global] youtrack.username <username>
          }
          die "Aborting."
        end

        username
      end

      def password
        password = `git config youtrack.password`.strip

        if password.empty?
          puts Term::ANSIColor.yellow %Q{
            Can't find Youtrack password!
            Please set it with:
            $ git config [--global] youtrack.password <password>
          }
          die "Aborting."
        end

        password
      end

      def url
        url = `git config youtrack.url`.strip

        if url.empty?
          puts Term::ANSIColor.yellow %Q{
            Can't find Youtrack URL!
            Please set it with:
            $ git config [--global] youtrack.url <https://mydomain.youtrack.net>
          }
          die "Aborting."
        end

        url
      end
    end
  end
end
