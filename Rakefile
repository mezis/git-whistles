#!/usr/bin/env rake
# encoding: UTF-8
require "bundler/gem_tasks"
require 'rspec/core/rake_task'
require 'pathname'

RSpec::Core::RakeTask.new(:spec)
task :default => :spec

task :binstubs do
  Pathname.glob('libexec/*.sh').each do |script|
    name = script.basename('.sh').to_s
    next if name == 'git-whistles'

    puts "bin/git-whistles -> bin/#{name}"
    FileUtils.remove_file("bin/#{name}")
    FileUtils.copy_file('bin/git-whistles', "bin/#{name}")
  end
end
