#!/usr/bin/env ruby
# encoding: UTF-8
#
# runner.rb --
#
#   Run shell scripts from a gem.
#   Symlink to this script to run a script with the same name living in
#   libexec/.
#
require 'pathname'
require 'rubygems'
require 'git-whistles'

target_script = Pathname.new($0).basename
script_path = Git::Whistles::GEMDIR.join('libexec', "#{target_script}.sh").cleanpath.to_s

Kernel.exec script_path, *ARGV
