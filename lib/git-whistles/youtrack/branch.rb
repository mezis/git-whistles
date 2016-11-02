require_relative '../app'
require_relative '../youtrack/api'
require_relative '../youtrack/ticket'

module Git::Whistles
  module Youtrack
    class Branch < Git::Whistles::App

      def initialize(current_dir: ENV['PWD'])
        @current_dir = current_dir
        super()
      end

      def main(args)
        super

        parse_args!(args)
        if args.count < 1
          puts Term::ANSIColor.yellow usage
          return false
        end

        ticket = get_ticket_from args[0]
        if ticket.nil?
          die "Could not find ticket #{args[0]}"
          return false
        end

        suggested_branch_name = suggested_branch_name_from(ticket)
        print_suggested_branch suggested_branch_name

        user_branch_name = read_user_answer
        show_and_exit "Cancelled by user" if user_branch_name.nil?

        final_branch_name = user_branch_name.empty? ? suggested_branch_name : user_branch_name
        create_branch final_branch_name

        puts Term::ANSIColor.green 'Created branch'
        true
      end

      private

      def usage
        'Usage: git youtrack-branch YOUTRACK_STORY_ID'
      end

      def get_ticket_from ticket_id
        youtrack_client.find_ticket ticket_id
      end

      def youtrack_client
        @youtrack_client ||= Git::Whistles::Youtrack::Api.new
      end

      def suggested_branch_name_from ticket
        "#{ticket.project}/#{ticket.title}-#{ticket.id}".
          downcase.
          gsub(/[^\w\d\/]/, '-').gsub!(/-+/, '-')
      end

      def print_suggested_branch branch_name
        puts 'The suggested branch name is: ' << Term::ANSIColor.yellow(branch_name)
        puts '    Press ENTER if you agree'
        puts ' or Write any other name and press ENTER'
        puts ' or Press CTRL-D to cancel'
      end

      def read_user_answer
        Readline.readline '  > ', false
      end

      def create_branch branch_name
        `cd #{@current_dir} && git checkout -b #{branch_name}`
      end
    end
  end
end
