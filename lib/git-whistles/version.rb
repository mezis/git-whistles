require 'pathname'

module Git
  module Whistles
    VERSION = "0.5.1"
    GEMDIR = Pathname.new(__FILE__).parent.parent.parent
  end
end
