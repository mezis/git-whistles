require 'tempfile'
require 'pry'

def example_title
  RSpec.current_example.description
end

# a trivial PO file loader. returns a hash.
def load_po(data)
  h = {}
  state = :none # :id, :str
  id = ""
  str = ""
  (data.split("\n") + ['']).each do |line|

    case state
    when :none
      next if line =~ /^$|^#/
      fail unless line =~ /^msgid "(.*)"$/
      id = $1
      state = :id
    when :id
      if line =~ /^msgstr "(.*)"/
        state = :str
        str = $1
        next
      end
      fail unless line =~ /^"(.*)"$/
      id += $1
    when :str
      if line =~ /^$|^#/
        state = :none
        h[id] = str
        next
      end
      fail unless line =~ /^"(.*)"$/
      str += $1
    end
  end
  h
end

describe 'git merge-po' do
  subject do
    f_local  = Tempfile.new('spec')
    f_base   = Tempfile.new('spec')
    f_remote = Tempfile.new('spec')

    f_local.write  File.read('spec/data/local.po')
    f_base.write   File.read('spec/data/base.po')
    f_remote.write File.read('spec/data/remote.po')

    f_local.flush
    f_base.flush
    f_remote.flush

    system "./libexec/git-merge-po.sh #{f_base.path} #{f_local.path} #{f_remote.path}"

    data = f_local.tap(&:rewind).read
    # Uncomment to get a readable output file:
    # File.open('spec/data/output.po', 'w') { |io| io.write data }
    
    load_po(data)
  end

  {
    "This little piggie is unchanged"                                 => '1',
    "This little piggie is removed from remote, unchanged on local"   => '2',
    "This little piggie is removed from remote, changed on local"     => '3.local',
    "This little piggie is removed from local, unchanged on remote"   => '4',
    "This little piggie is removed from local, changed on remote"     => '5.remote',
    "This little piggie is changed on local, unchanged on remote"     => '6.local',
    "This little piggie is changed on remote, unchanged on local"     => '7.remote',
    "This little piggie is added on remote, not on local"             => '9.remote',
    "This little piggie is added on local, not on remote"             => '10.local'
  }.each_pair do |piggie, value|
    it piggie do
      expect(subject[piggie]).to eq(value)
    end
  end

  it 'This little piggie is changed on remote and local' do
    value = subject[example_title]
    expect(value).to include('8.local')
    expect(value).to include('8.remote')
    expect(value).to include('#-#-#')
  end

  it 'This little piggie is added on local and remote, with different values' do
    value = subject[example_title]
    expect(value).to include('11.local')
    expect(value).to include('11.remote')
    expect(value).to include('#-#-#')
  end

end
