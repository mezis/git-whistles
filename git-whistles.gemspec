# -*- encoding: utf-8 -*-

Gem::Specification.new do |s|
  s.name        = "git-whistles"
  s.version     = '0.1.0'
  s.platform    = Gem::Platform::RUBY
  s.authors     = ["Julien Letessier"]
  s.email       = ["julien.letessier@gmail.com"]
  s.homepage    = "https://github.com/mezus/git-whistles"
  s.summary     = "A few bells and whistles for Git"
  s.description = "A few bells and whistles for Git: chop, list-branches, merge-po, stash-and-checkout"

  s.required_rubygems_version = ">= 1.3.6"

  s.files        = `git ls-files`.split("\n")
  s.test_files   = []
  s.executables  = `git ls-files`.split("\n").map{|f| f =~ /^bin\/(.*)/ ? $1 : nil}.compact
  s.require_path = nil
end
