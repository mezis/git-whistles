#!/usr/bin/env ruby
#
# runner.rb --
#
#   Run shell scripts from a gem.
#
require 'pathname'

caller_path = Pathname.new($0)
script_path = caller_path.join('../../libexec', "#{caller_path.basename}.sh").cleanpath

Kernel.exec script_path, *ARGV
