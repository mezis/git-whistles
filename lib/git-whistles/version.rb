require 'pathname'

module Git
  module Whistles
    VERSION = "0.4.3"
    GEMDIR = Pathname.new(__FILE__).parent.parent.parent
  end
end
