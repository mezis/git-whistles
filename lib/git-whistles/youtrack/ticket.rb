module Git::Whistles
  module Youtrack
    class Ticket
      class << self
        def build_from_remote(ticket_hash)
          return nil if ticket_hash.nil?
          return nil if issue_not_found? ticket_hash

          self.new.tap do |ticket|
            ticket.title = title_from ticket_hash
            ticket.id = id_from ticket_hash
            ticket.project = project_from ticket_hash
            ticket.description = description_from ticket_hash
          end
        end

        def title_from ticket_hash
          ticket_hash.
            dig('issue', 'field').
            select { |f| f['name'] == 'summary'  }.
            first.dig('value')
        end

        def id_from ticket_hash
          ticket_hash.
            dig('issue', 'field').
            select { |f| f['name'] == 'numberInProject'  }.
            first.dig('value').to_i
        end

        def project_from ticket_hash
          ticket_hash.
            dig('issue', 'field').
            select { |f| f['name'] == 'projectShortName'  }.
            first.dig('value')
        end

        def description_from ticket_hash
          ticket_hash.
            dig('issue', 'field').
            select { |f| f['name'] == 'description'  }.
            first.dig('value')
        end

        def issue_not_found? ticket_hash
          if ticket_hash["error"]
            true
          else
            false
          end
        end
      end

      attr_accessor :title
      attr_accessor :id
      attr_accessor :project
      attr_accessor :description
    end
  end
end