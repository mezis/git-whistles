require 'spec_helper'
require 'pry'
require 'webmock'
require 'vcr'
require_relative '../../../../lib/git-whistles/youtrack/api'

describe Git::Whistles::Youtrack::Api do
  before do
    expect(subject).to receive(:`).
      with('git config youtrack.username').
      and_return('some_username')

    expect(subject).to receive(:`).
      with('git config youtrack.password').
      and_return('some_password')

    expect(subject).to receive(:`).
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
end
