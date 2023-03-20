{ ... }:

{
  # https://devenv.sh/languages/
  languages.nix.enable = true;

  languages.rust = {
    enable = true;
    version = "stable";
  };
}
