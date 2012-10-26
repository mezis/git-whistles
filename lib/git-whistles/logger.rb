require 'logger'

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
      Term::ANSIColor.send(Colors[severity], "#{msg}\n")
    end
  end
end