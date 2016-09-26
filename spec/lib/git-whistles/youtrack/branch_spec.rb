require 'spec_helper'
require 'pry'
require_relative '../../../../lib/git-whistles/youtrack/branch'

RSpec.describe Git::Whistles::Youtrack::Branch do
  describe '#main' do
    subject { described_class.new(current_dir: '/home/myuser') }

    context 'with no arguments' do
      it 'prints an error message' do
        expect(subject).to receive(:puts).with("Usage: git youtrack-branch YOUTRACK_STORY_ID")

        subject.main([])
      end

      it 'returns false' do
        expect(subject).to receive(:puts)

        expect(subject.main([])).to eq(false)
      end
    end

    context 'with a ticket id argument' do
      before do
        expect(Readline).to receive(:readline).and_return ""
        expect(Git::Whistles::Youtrack::Api).to receive(:new).and_return youtrack_api
        expect(subject).to receive(:puts).at_least(:once)
      end

      let(:youtrack_api) { double(get_ticket: ticket) }
      let(:ticket_id) { "338" }
      let(:ticket) do
        Git::Whistles::Youtrack::Ticket.new.tap do |ticket|
          ticket.title       = "Some title"
          ticket.project     = "PRJ"
          ticket.id          = ticket_id.to_i
          ticket.description = "Some description"
        end
      end

      it 'issues a neat checkout command' do
        expect(subject).to receive(:`).with('cd /home/myuser && git checkout -b prj/some-title-338')

        subject.main([ticket_id])
      end

      it 'returns true' do
        expect(subject).to receive(:`)

        expect(subject.main([ticket_id])).to eq(true)
      end
    end
  end
end