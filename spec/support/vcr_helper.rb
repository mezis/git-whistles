require 'vcr'

VCR.configure do |config|
  config.cassette_library_dir = "spec/fixtures/vcr"
  config.hook_into :webmock
  # config.debug_logger = $stdout
  config.ignore_localhost = true
end
