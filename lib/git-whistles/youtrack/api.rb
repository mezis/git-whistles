require 'youtrack'
require 'term/ansicolor'
require_relative '../../git-whistles'
require_relative 'ticket'

module Git::Whistles
  module Youtrack
    class Api
      def find_ticket(id)
        ticket_hash = find_issue(id)
        Ticket.build_from_remote(ticket_hash)
      end

      def username
        username = `git config youtrack.username`.strip

        if username.empty?
          puts Term::ANSIColor.yellow %Q{
            Can't find Youtrack username!
            Please set it with:
            $ git config [--global] youtrack.username <username>
          }
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
        end

        url
      end

      private

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

      def find_issue id
        issues.find(id)
      rescue
        nil
      end
    end
  end
end
