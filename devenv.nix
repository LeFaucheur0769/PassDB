{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.cargo
    pkgs.python314
    pkgs.python314Packages.pyyaml
    pkgs.python314Packages.pip
    pkgs.python314Packages.questionary
    pkgs.python314Packages.pyside6
    pkgs.python314Packages.pyqtdarktheme
    pkgs.qt6.qtbase
    pkgs.qt6.qtsvg
    pkgs.qt6.qttools
    pkgs.qt6.qtdeclarative
    pkgs.nerd-fonts.fira-code
    pkgs.python314Packages.black

  ];

  #https://devenv.sh/languages/
  languages.rust.enable = true;
  languages.python.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts = {
    hello.exec = ''
      echo hello from $GREET
    '';
  };

  # https://devenv.sh/basics/
  enterShell = ''
    fish
      hello         # Run scripts directly
      git --version # Use packages
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
     echo "Running tests"
    python GUI/gui.py 
  '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
