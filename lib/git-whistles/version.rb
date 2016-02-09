# encoding: UTF-8

require 'pathname'

module Git
  module Whistles
    VERSION = "1.0.2"
    GEMDIR = Pathname.new(__FILE__).parent.parent.parent
  end
end
