require 'logger'
require 'git-whistles'

module Git::Whistles
  class Logger < ::Logger
    Colors = {
      'DEBUG'   => :reset,
      'INFO'    => :green,
      'WARN'    => :yellow,
      'ERROR'   => :red,
      'FATAL'   => :red,
      'UNKNOWN' => :red
    }
    
    def initialize(*args)
      super
      self.formatter = self.method(:custom_formatter)
    end

    def custom_formatter(severity, time, progname, msg)
      msg = msg.sub(/([^¬])$/,"\\1\n").sub(/¬$/,'')
      Term::ANSIColor.send(Colors[severity], msg)
    end
  end
end