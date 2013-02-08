# encoding: UTF-8
require 'ostruct'
require 'optparse'
require 'term/ansicolor'
require 'git-whistles/logger'


module Git::Whistles
  class App

    def initialize
      @options = OpenStruct.new(defaults)
      @log = Git::Whistles::Logger.new($stderr)
    end


    def main(args)
      parse_args!(args)
    end


    def self.run!
      new.main(ARGV)
    end


    protected

    # default options hash
    def defaults
      {}
    end

    attr :options
    attr :log


    def option_parser
      @option_parser ||= OptionParser.new do |op|
        op.banner = "Usage: #{$0}"
      end
    end



    def parse_args!(args)
      begin
        option_parser.parse!(args)
      rescue OptionParser::InvalidOption => error
        die error.message, :usage => true
      end
    end


    def run(command)
      %x(#{command})
    end


    def run!(command)
      result = %x(#{command})
      return result if $? == 0
      die "command '#{command}' failed"
    end


    def die(message, options = {})
      puts Term::ANSIColor.red(message)
      if options[:usage]
        puts 
        puts option_parser.help
      end
      exit 1
    end
  end
end
