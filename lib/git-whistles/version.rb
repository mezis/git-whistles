require 'pathname'

module Git
  module Whistles
    VERSION = "0.6.0"
    GEMDIR = Pathname.new(__FILE__).parent.parent.parent
  end
end
