cask "oxide" do
  version "1.0.0-nightly.20261014.58"
  sha256 "efbaf7adac3075c639249d387aa419bf44fe39ab84c079e4080cd6b0a228ed65"

  url "https://github.com/yksaionara/oxide/releases/download/v#{version}/oxide-v#{version}-macOS-ARM64.zip"
  name "Oxide"
  desc "Experimental native Discord client"
  homepage "https://github.com/yksaionara/oxide"

  # Frozen: this cask moved to the yksaionara/homebrew-oxide tap and is no longer updated here.
  deprecate! date: "2026-10-16",
             because: "moved to its own tap; run `brew untap yksaionara/oxide && brew tap yksaionara/oxide`"

  depends_on arch: :arm64
  depends_on macos: :sonoma

  app "Oxide.app"
end
