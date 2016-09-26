require 'spec_helper'
require 'pry'
require 'webmock'
require 'vcr'
require_relative '../../../../lib/git-whistles/youtrack/api'

describe Git::Whistles::Youtrack::Api do
  before do
    allow(subject).to receive(:`).
      with('git config youtrack.username').
      and_return('some_username')

    allow(subject).to receive(:`).
      with('git config youtrack.password').
      and_return('some_password')

    allow(subject).to receive(:`).
      with('git config youtrack.url').
      and_return('http://youtrack.com/')
  end

  describe '#username' do
    it 'returns the youtrack api username' do
      expect(subject.username).to eq('some_username')
    end
  end

  describe '#password' do
    it 'returns the youtrack api password' do
      expect(subject.password).to eq('some_password')
    end
  end

  describe '#get_ticket' do
    context 'when successfull' do
      it 'returns a ticket with title' do
        VCR.use_cassette('get_ticket_successfull') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket.title).to eq("Header styling changes")
        end
      end

      it 'returns a ticket with id' do
        VCR.use_cassette('get_ticket_successfull') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket.id).to eq(338)
        end
      end

      it 'returns a ticket project name' do
        VCR.use_cassette('get_ticket_successfull') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket.project).to eq("PIO")
        end
      end

      it 'returns a ticket description' do
        VCR.use_cassette('get_ticket_successfull') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket.description).to include("==Why==")
        end
      end
    end

    context 'when ticket not found' do
      it 'returns nil' do
        VCR.use_cassette('get_ticket_not_found') do
          ticket = subject.get_ticket('non-existent')

          expect(ticket).to be_nil
        end
      end
    end

    context 'when authentication fails' do
      it 'returns nil' do
        VCR.use_cassette('get_ticket_auth_failure') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket).to be_nil
        end
      end
    end

    context 'when credentials not setup' do
      before do
        allow(subject).to receive(:`).
          with('git config youtrack.username').
          and_return('')

        allow(Term::ANSIColor).to receive(:yellow).and_return "warning for user"
      end

      it 'prints a message and returns nil' do
        expect(subject).to receive(:puts).with "warning for user"

        VCR.use_cassette('get_ticket_auth_failure') do
          ticket = subject.get_ticket('PIO-338')

          expect(ticket).to be_nil
        end
      end
    end
  end
end
