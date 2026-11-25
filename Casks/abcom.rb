cask "abcom" do
  version "1.0.0-beta.3"
  sha256 "1b42a448d7007117fb77f5e17a9adf6facb834c5b12412f61aa400950bfc3a59"

  url "https://github.com/abnd-corp/Abcom/releases/download/v1.0.0-beta.3/abcom-macos-aarch64.tar.gz"
  name "Abcom"
  desc "Encrypted peer-to-peer LAN messenger"
  homepage "https://github.com/abnd-corp/Abcom"

  depends_on arch: :arm64

  app "Abcom.app"

  uninstall launchctl: "Abcom",
            quit: "corp.abnd.abcom",
            delete: "~/Library/LaunchAgents/Abcom.plist"
end
