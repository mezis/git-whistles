# -*- encoding: utf-8 -*-
require File.expand_path('../lib/git-whistles/version', __FILE__)

Gem::Specification.new do |gem|
  gem.authors       = ["Julien Letessier"]
  gem.email         = ["julien.letessier@gmail.com"]
  gem.description   = %q{A few helpers for classic Git workflows}
  gem.summary       = %q{
    A few helpers for classic Git workflows:
    makes branching and merging, PO file handling, issuing pull requests
    slightly simpler.
  }
  gem.homepage      = "http://github.com/mezis/git-whistles"

  gem.required_rubygems_version = ">= 1.3.6"

  gem.add_development_dependency "bundler", ">= 1.0.0"
  gem.add_development_dependency "rake"

  gem.files         = `git ls-files`.split($\)
  gem.executables   = gem.files.grep(%r{^bin/}).map{ |f| File.basename(f) }
  gem.test_files    = gem.files.grep(%r{^(test|spec|features)/})
  gem.name          = "git-whistles"
  gem.require_paths = ["lib"]
  gem.version       = Git::Whistles::VERSION
end
