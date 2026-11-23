cask "abcom" do
  version "1.0.0-beta.2"
  sha256 "887d1494486bb5420cf364f3718a2ae480f635f3318e69facc48a0eb7739d832"

  url "https://github.com/abnd-corp/Abcom/releases/download/v1.0.0-beta.2/abcom-macos-aarch64.tar.gz"
  name "Abcom"
  desc "Encrypted peer-to-peer LAN messenger"
  homepage "https://github.com/abnd-corp/Abcom"

  depends_on arch: :arm64

  app "Abcom.app"

  uninstall launchctl: "Abcom",
            quit: "corp.abnd.abcom",
            delete: "~/Library/LaunchAgents/Abcom.plist"
end
