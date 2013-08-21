# encoding: UTF-8

require 'pathname'

module Git
  module Whistles
    VERSION = "0.7.6"
    GEMDIR = Pathname.new(__FILE__).parent.parent.parent
  end
end
