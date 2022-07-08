
# This file was @generated by crate2nix 0.10.0 with the command:
#   "generate" "--no-default-features"
# See https://github.com/kolloch/crate2nix for more info.

{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs { config = {}; }
, lib ? pkgs.lib
, stdenv ? pkgs.stdenv
, buildRustCrateForPkgs ? if buildRustCrate != null
    then lib.warn "crate2nix: Passing `buildRustCrate` as argument to Cargo.nix is deprecated. If you don't customize `buildRustCrate`, replace `callPackage ./Cargo.nix {}` by `import ./Cargo.nix { inherit pkgs; }`, and if you need to customize `buildRustCrate`, use `buildRustCrateForPkgs` instead." (_: buildRustCrate)
    else pkgs: pkgs.buildRustCrate
  # Deprecated
, buildRustCrate ? null
  # This is used as the `crateOverrides` argument for `buildRustCrate`.
, defaultCrateOverrides ? pkgs.defaultCrateOverrides
  # The features to enable for the root_crate or the workspace_members.
, rootFeatures ? [ "default" ]
  # If true, throw errors instead of issueing deprecation warnings.
, strictDeprecation ? false
  # Used for conditional compilation based on CPU feature detection.
, targetFeatures ? []
  # Whether to perform release builds: longer compile times, faster binaries.
, release ? true
  # Additional crate2nix configuration if it exists.
, crateConfig
  ? if builtins.pathExists ./crate-config.nix
    then pkgs.callPackage ./crate-config.nix {}
    else {}
}:

rec {
  #
  # "public" attributes that we attempt to keep stable with new versions of crate2nix.
  #

  rootCrate = rec {
    packageId = "toml-editor";

    # Use this attribute to refer to the derivation building your root crate package.
    # You can override the features with rootCrate.build.override { features = [ "default" "feature1" ... ]; }.
    build = internal.buildRustCrateWithFeatures {
      inherit packageId;
    };

    # Debug support which might change between releases.
    # File a bug if you depend on any for non-debug work!
    debug = internal.debugCrate { inherit packageId; };
  };
  # Refer your crate build derivation by name here.
  # You can override the features with
  # workspaceMembers."${crateName}".build.override { features = [ "default" "feature1" ... ]; }.
  workspaceMembers = {
    "toml-editor" = rec {
      packageId = "toml-editor";
      build = internal.buildRustCrateWithFeatures {
        packageId = "toml-editor";
      };

      # Debug support which might change between releases.
      # File a bug if you depend on any for non-debug work!
      debug = internal.debugCrate { inherit packageId; };
    };
  };

  # A derivation that joins the outputs of all workspace members together.
  allWorkspaceMembers = pkgs.symlinkJoin {
      name = "all-workspace-members";
      paths =
        let members = builtins.attrValues workspaceMembers;
        in builtins.map (m: m.build) members;
  };

  #
  # "internal" ("private") attributes that may change in every new version of crate2nix.
  #

  internal = rec {
    # Build and dependency information for crates.
    # Many of the fields are passed one-to-one to buildRustCrate.
    #
    # Noteworthy:
    # * `dependencies`/`buildDependencies`: similar to the corresponding fields for buildRustCrate.
    #   but with additional information which is used during dependency/feature resolution.
    # * `resolvedDependencies`: the selected default features reported by cargo - only included for debugging.
    # * `devDependencies` as of now not used by `buildRustCrate` but used to
    #   inject test dependencies into the build

    crates = {
      "autocfg" = rec {
        crateName = "autocfg";
        version = "1.1.0";
        edition = "2015";
        sha256 = "1ylp3cb47ylzabimazvbz9ms6ap784zhb6syaz6c1jqpmcmq0s6l";
        authors = [
          "Josh Stone <cuviper@gmail.com>"
        ];

      };
      "bytes" = rec {
        crateName = "bytes";
        version = "1.1.0";
        edition = "2018";
        sha256 = "1y70b249m02lfp0j6565b29kviapj4xsl9whamcqwddnp9kjv1y4";
        authors = [
          "Carl Lerche <me@carllerche.com>"
          "Sean McArthur <sean@seanmonstar.com>"
        ];
        features = {
          "default" = [ "std" ];
        };
        resolvedDefaultFeatures = [ "default" "std" ];
      };
      "combine" = rec {
        crateName = "combine";
        version = "4.6.4";
        edition = "2018";
        sha256 = "0j69sz8v8pxz9id0yqp7c5jfd7fnyak8bjkgg8r0h64xny9lwq1a";
        authors = [
          "Markus Westerlind <marwes91@gmail.com>"
        ];
        dependencies = [
          {
            name = "bytes";
            packageId = "bytes";
            optional = true;
          }
          {
            name = "memchr";
            packageId = "memchr";
            usesDefaultFeatures = false;
          }
        ];
        devDependencies = [
          {
            name = "bytes";
            packageId = "bytes";
          }
        ];
        features = {
          "default" = [ "std" ];
          "futures-03" = [ "pin-project" "std" "futures-core-03" "futures-io-03" "pin-project-lite" ];
          "pin-project" = [ "pin-project-lite" ];
          "std" = [ "memchr/use_std" "bytes" "alloc" ];
          "tokio" = [ "tokio-dep" "tokio-util/io" "futures-core-03" "pin-project-lite" ];
          "tokio-02" = [ "pin-project" "std" "tokio-02-dep" "futures-core-03" "pin-project-lite" "bytes_05" ];
          "tokio-03" = [ "pin-project" "std" "tokio-03-dep" "futures-core-03" "pin-project-lite" ];
        };
        resolvedDefaultFeatures = [ "alloc" "bytes" "default" "std" ];
      };
      "either" = rec {
        crateName = "either";
        version = "1.6.1";
        edition = "2015";
        sha256 = "0mwl9vngqf5jvrhmhn9x60kr5hivxyjxbmby2pybncxfqhf4z3g7";
        authors = [
          "bluss"
        ];
        features = {
          "default" = [ "use_std" ];
        };
      };
      "hashbrown" = rec {
        crateName = "hashbrown";
        version = "0.11.2";
        edition = "2018";
        sha256 = "0vkjsf5nzs7qcia5ya79j9sq2p1caz4crrncr1675wwyj3ag0pmb";
        authors = [
          "Amanieu d'Antras <amanieu@gmail.com>"
        ];
        features = {
          "ahash-compile-time-rng" = [ "ahash/compile-time-rng" ];
          "default" = [ "ahash" "inline-more" ];
          "rustc-dep-of-std" = [ "nightly" "core" "compiler_builtins" "alloc" "rustc-internal-api" ];
        };
        resolvedDefaultFeatures = [ "raw" ];
      };
      "indexmap" = rec {
        crateName = "indexmap";
        version = "1.8.2";
        edition = "2018";
        sha256 = "0nnaw0whv3xysrpjrz69bsibbscd81rwx63s6f4kbajv1ia2s0g6";
        authors = [
          "bluss"
          "Josh Stone <cuviper@gmail.com>"
        ];
        dependencies = [
          {
            name = "hashbrown";
            packageId = "hashbrown";
            usesDefaultFeatures = false;
            features = [ "raw" ];
          }
        ];
        buildDependencies = [
          {
            name = "autocfg";
            packageId = "autocfg";
          }
        ];
        features = {
          "serde-1" = [ "serde" ];
        };
      };
      "itertools" = rec {
        crateName = "itertools";
        version = "0.10.3";
        edition = "2018";
        sha256 = "1qy55fqbaisr9qgbn7cvdvqlfqbh1f4ddf99zwan56z7l6gx3ad9";
        authors = [
          "bluss"
        ];
        dependencies = [
          {
            name = "either";
            packageId = "either";
            usesDefaultFeatures = false;
          }
        ];
        features = {
          "default" = [ "use_std" ];
          "use_std" = [ "use_alloc" ];
        };
        resolvedDefaultFeatures = [ "default" "use_alloc" "use_std" ];
      };
      "itoa" = rec {
        crateName = "itoa";
        version = "1.0.2";
        edition = "2018";
        sha256 = "13ap85z7slvma9c36bzi7h5j66dm5sxm4a2g7wiwxbsh826nfb0i";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
        ];

      };
      "kstring" = rec {
        crateName = "kstring";
        version = "1.0.6";
        edition = "2018";
        sha256 = "09j5xb3rnjd3kmc2v667wzsc4mz4c1hl1vkzszbj30fyxb60qccb";
        authors = [
          "Ed Page <eopage@gmail.com>"
        ];
        dependencies = [
          {
            name = "serde";
            packageId = "serde";
            optional = true;
          }
        ];
        features = {
          "default" = [ "serde" ];
        };
        resolvedDefaultFeatures = [ "default" "max_inline" "serde" ];
      };
      "memchr" = rec {
        crateName = "memchr";
        version = "2.5.0";
        edition = "2018";
        sha256 = "0vanfk5mzs1g1syqnj03q8n0syggnhn55dq535h2wxr7rwpfbzrd";
        authors = [
          "Andrew Gallant <jamslam@gmail.com>"
          "bluss"
        ];
        features = {
          "default" = [ "std" ];
          "rustc-dep-of-std" = [ "core" "compiler_builtins" ];
          "use_std" = [ "std" ];
        };
        resolvedDefaultFeatures = [ "std" "use_std" ];
      };
      "proc-macro2" = rec {
        crateName = "proc-macro2";
        version = "1.0.39";
        edition = "2018";
        sha256 = "0vzm2m7rq6sym9w73ca3hpc5m9wkwm500hyya6bgrdr5j1b2ajy5";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
          "Alex Crichton <alex@alexcrichton.com>"
        ];
        dependencies = [
          {
            name = "unicode-ident";
            packageId = "unicode-ident";
          }
        ];
        features = {
          "default" = [ "proc-macro" ];
        };
        resolvedDefaultFeatures = [ "default" "proc-macro" ];
      };
      "quote" = rec {
        crateName = "quote";
        version = "1.0.18";
        edition = "2018";
        sha256 = "1lca4xnwdc2sp76bf4n50kifmi5phhxr9520w623mfcksr7bbzm1";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
        ];
        dependencies = [
          {
            name = "proc-macro2";
            packageId = "proc-macro2";
            usesDefaultFeatures = false;
          }
        ];
        features = {
          "default" = [ "proc-macro" ];
          "proc-macro" = [ "proc-macro2/proc-macro" ];
        };
        resolvedDefaultFeatures = [ "default" "proc-macro" ];
      };
      "ryu" = rec {
        crateName = "ryu";
        version = "1.0.10";
        edition = "2018";
        sha256 = "15960rzj6jkjhxrjfr3kid2hbnia84s6h8l1ga7vkla9rwmgkxpk";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
        ];
        features = {
        };
      };
      "serde" = rec {
        crateName = "serde";
        version = "1.0.137";
        edition = "2015";
        sha256 = "1l8pynxnmld179a33l044yvkigq3fhiwgx0518a1b0vzqxa8vsk1";
        authors = [
          "Erick Tryzelaar <erick.tryzelaar@gmail.com>"
          "David Tolnay <dtolnay@gmail.com>"
        ];
        dependencies = [
          {
            name = "serde_derive";
            packageId = "serde_derive";
            optional = true;
          }
        ];
        devDependencies = [
          {
            name = "serde_derive";
            packageId = "serde_derive";
          }
        ];
        features = {
          "default" = [ "std" ];
          "derive" = [ "serde_derive" ];
        };
        resolvedDefaultFeatures = [ "default" "derive" "serde_derive" "std" ];
      };
      "serde_derive" = rec {
        crateName = "serde_derive";
        version = "1.0.137";
        edition = "2015";
        sha256 = "1gkqhpw86zvppd0lwa8ljzpglwczxq3d7cnkfwirfn9r1jxgl9hz";
        procMacro = true;
        authors = [
          "Erick Tryzelaar <erick.tryzelaar@gmail.com>"
          "David Tolnay <dtolnay@gmail.com>"
        ];
        dependencies = [
          {
            name = "proc-macro2";
            packageId = "proc-macro2";
          }
          {
            name = "quote";
            packageId = "quote";
          }
          {
            name = "syn";
            packageId = "syn";
          }
        ];
        features = {
        };
        resolvedDefaultFeatures = [ "default" ];
      };
      "serde_json" = rec {
        crateName = "serde_json";
        version = "1.0.81";
        edition = "2018";
        sha256 = "0p7100hlvw4azgcalzf1vgray5cg6b6saqfwb32h7v8s5ary4z4v";
        authors = [
          "Erick Tryzelaar <erick.tryzelaar@gmail.com>"
          "David Tolnay <dtolnay@gmail.com>"
        ];
        dependencies = [
          {
            name = "itoa";
            packageId = "itoa";
          }
          {
            name = "ryu";
            packageId = "ryu";
          }
          {
            name = "serde";
            packageId = "serde";
            usesDefaultFeatures = false;
          }
        ];
        devDependencies = [
          {
            name = "serde";
            packageId = "serde";
            features = [ "derive" ];
          }
        ];
        features = {
          "alloc" = [ "serde/alloc" ];
          "default" = [ "std" ];
          "preserve_order" = [ "indexmap" "std" ];
          "std" = [ "serde/std" ];
        };
        resolvedDefaultFeatures = [ "default" "std" ];
      };
      "syn" = rec {
        crateName = "syn";
        version = "1.0.96";
        edition = "2018";
        sha256 = "1gqymymz4202nfj76dkhr177wmcidch580vzf6w3qi943qjxsj07";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
        ];
        dependencies = [
          {
            name = "proc-macro2";
            packageId = "proc-macro2";
            usesDefaultFeatures = false;
          }
          {
            name = "quote";
            packageId = "quote";
            optional = true;
            usesDefaultFeatures = false;
          }
          {
            name = "unicode-ident";
            packageId = "unicode-ident";
          }
        ];
        features = {
          "default" = [ "derive" "parsing" "printing" "clone-impls" "proc-macro" ];
          "printing" = [ "quote" ];
          "proc-macro" = [ "proc-macro2/proc-macro" "quote/proc-macro" ];
          "test" = [ "syn-test-suite/all-features" ];
        };
        resolvedDefaultFeatures = [ "clone-impls" "default" "derive" "parsing" "printing" "proc-macro" "quote" ];
      };
      "toml-editor" = rec {
        crateName = "toml-editor";
        version = "0.4.2";
        edition = "2018";
        crateBin = [
          { name = "toml-editor"; path = "src/main.rs"; }
        ];
        src = lib.cleanSourceWith { filter = sourceFilter;  src = ./.; };
        dependencies = [
          {
            name = "serde";
            packageId = "serde";
            features = [ "derive" ];
          }
          {
            name = "serde_json";
            packageId = "serde_json";
          }
          {
            name = "toml_edit";
            packageId = "toml_edit";
          }
        ];

      };
      "toml_edit" = rec {
        crateName = "toml_edit";
        version = "0.10.1";
        edition = "2018";
        sha256 = "1a6zw9rd8m0qbal6a3dyq1mvyb9hv16ydsrvsrk6yzwgqh0yp0i7";
        authors = [
          "Andronik Ordian <write@reusable.software>"
          "Ed Page <eopage@gmail.com>"
        ];
        dependencies = [
          {
            name = "combine";
            packageId = "combine";
          }
          {
            name = "indexmap";
            packageId = "indexmap";
          }
          {
            name = "itertools";
            packageId = "itertools";
          }
          {
            name = "kstring";
            packageId = "kstring";
            features = [ "max_inline" ];
          }
        ];
        features = {
          "easy" = [ "serde" ];
        };
        resolvedDefaultFeatures = [ "default" ];
      };
      "unicode-ident" = rec {
        crateName = "unicode-ident";
        version = "1.0.1";
        edition = "2018";
        sha256 = "131niycgp77aiwvgjdyh47389xfnb7fmlc8ybrxys8v0a0kgxljv";
        authors = [
          "David Tolnay <dtolnay@gmail.com>"
        ];

      };
    };

    #
# crate2nix/default.nix (excerpt start)
#

  /* Target (platform) data for conditional dependencies.
    This corresponds roughly to what buildRustCrate is setting.
  */
  defaultTarget = {
    unix = true;
    windows = false;
    fuchsia = true;
    test = false;

    # This doesn't appear to be officially documented anywhere yet.
    # See https://github.com/rust-lang-nursery/rust-forge/issues/101.
    os =
      if stdenv.hostPlatform.isDarwin
      then "macos"
      else stdenv.hostPlatform.parsed.kernel.name;
    arch = stdenv.hostPlatform.parsed.cpu.name;
    family = "unix";
    env = "gnu";
    endian =
      if stdenv.hostPlatform.parsed.cpu.significantByte.name == "littleEndian"
      then "little" else "big";
    pointer_width = toString stdenv.hostPlatform.parsed.cpu.bits;
    vendor = stdenv.hostPlatform.parsed.vendor.name;
    debug_assertions = false;
  };

  /* Filters common temp files and build files. */
  # TODO(pkolloch): Substitute with gitignore filter
  sourceFilter = name: type:
    let
      baseName = builtins.baseNameOf (builtins.toString name);
    in
      ! (
        # Filter out git
        baseName == ".gitignore"
        || (type == "directory" && baseName == ".git")

        # Filter out build results
        || (
          type == "directory" && (
            baseName == "target"
            || baseName == "_site"
            || baseName == ".sass-cache"
            || baseName == ".jekyll-metadata"
            || baseName == "build-artifacts"
          )
        )

        # Filter out nix-build result symlinks
        || (
          type == "symlink" && lib.hasPrefix "result" baseName
        )

        # Filter out IDE config
        || (
          type == "directory" && (
            baseName == ".idea" || baseName == ".vscode"
          )
        ) || lib.hasSuffix ".iml" baseName

        # Filter out nix build files
        || baseName == "Cargo.nix"

        # Filter out editor backup / swap files.
        || lib.hasSuffix "~" baseName
        || builtins.match "^\\.sw[a-z]$$" baseName != null
        || builtins.match "^\\..*\\.sw[a-z]$$" baseName != null
        || lib.hasSuffix ".tmp" baseName
        || lib.hasSuffix ".bak" baseName
        || baseName == "tests.nix"
      );

  /* Returns a crate which depends on successful test execution
    of crate given as the second argument.

    testCrateFlags: list of flags to pass to the test exectuable
    testInputs: list of packages that should be available during test execution
  */
  crateWithTest = { crate, testCrate, testCrateFlags, testInputs, testPreRun, testPostRun }:
    assert builtins.typeOf testCrateFlags == "list";
    assert builtins.typeOf testInputs == "list";
    assert builtins.typeOf testPreRun == "string";
    assert builtins.typeOf testPostRun == "string";
    let
      # override the `crate` so that it will build and execute tests instead of
      # building the actual lib and bin targets We just have to pass `--test`
      # to rustc and it will do the right thing.  We execute the tests and copy
      # their log and the test executables to $out for later inspection.
      test =
        let
          drv = testCrate.override
            (
              _: {
                buildTests = true;
              }
            );
          # If the user hasn't set any pre/post commands, we don't want to
          # insert empty lines. This means that any existing users of crate2nix
          # don't get a spurious rebuild unless they set these explicitly.
          testCommand = pkgs.lib.concatStringsSep "\n"
            (pkgs.lib.filter (s: s != "") [
              testPreRun
              "$f $testCrateFlags 2>&1 | tee -a $out"
              testPostRun
            ]);
        in
        pkgs.runCommand "run-tests-${testCrate.name}"
          {
            inherit testCrateFlags;
            buildInputs = testInputs;
          } ''
          set -ex

          export RUST_BACKTRACE=1

          # recreate a file hierarchy as when running tests with cargo

          # the source for test data
          ${pkgs.xorg.lndir}/bin/lndir ${crate.src}

          # build outputs
          testRoot=target/debug
          mkdir -p $testRoot

          # executables of the crate
          # we copy to prevent std::env::current_exe() to resolve to a store location
          for i in ${crate}/bin/*; do
            cp "$i" "$testRoot"
          done
          chmod +w -R .

          # test harness executables are suffixed with a hash, like cargo does
          # this allows to prevent name collision with the main
          # executables of the crate
          hash=$(basename $out)
          for file in ${drv}/tests/*; do
            f=$testRoot/$(basename $file)-$hash
            cp $file $f
            ${testCommand}
          done
        '';
    in
    pkgs.runCommand "${crate.name}-linked"
      {
        inherit (crate) outputs crateName;
        passthru = (crate.passthru or { }) // {
          inherit test;
        };
      } ''
      echo tested by ${test}
      ${lib.concatMapStringsSep "\n" (output: "ln -s ${crate.${output}} ${"$"}${output}") crate.outputs}
    '';

  /* A restricted overridable version of builtRustCratesWithFeatures. */
  buildRustCrateWithFeatures =
    { packageId
    , features ? rootFeatures
    , crateOverrides ? defaultCrateOverrides
    , buildRustCrateForPkgsFunc ? null
    , runTests ? false
    , testCrateFlags ? [ ]
    , testInputs ? [ ]
      # Any command to run immediatelly before a test is executed.
    , testPreRun ? ""
      # Any command run immediatelly after a test is executed.
    , testPostRun ? ""
    }:
    lib.makeOverridable
      (
        { features
        , crateOverrides
        , runTests
        , testCrateFlags
        , testInputs
        , testPreRun
        , testPostRun
        }:
        let
          buildRustCrateForPkgsFuncOverriden =
            if buildRustCrateForPkgsFunc != null
            then buildRustCrateForPkgsFunc
            else
              (
                if crateOverrides == pkgs.defaultCrateOverrides
                then buildRustCrateForPkgs
                else
                  pkgs: (buildRustCrateForPkgs pkgs).override {
                    defaultCrateOverrides = crateOverrides;
                  }
              );
          builtRustCrates = builtRustCratesWithFeatures {
            inherit packageId features;
            buildRustCrateForPkgsFunc = buildRustCrateForPkgsFuncOverriden;
            runTests = false;
          };
          builtTestRustCrates = builtRustCratesWithFeatures {
            inherit packageId features;
            buildRustCrateForPkgsFunc = buildRustCrateForPkgsFuncOverriden;
            runTests = true;
          };
          drv = builtRustCrates.crates.${packageId};
          testDrv = builtTestRustCrates.crates.${packageId};
          derivation =
            if runTests then
              crateWithTest
                {
                  crate = drv;
                  testCrate = testDrv;
                  inherit testCrateFlags testInputs testPreRun testPostRun;
                }
            else drv;
        in
        derivation
      )
      { inherit features crateOverrides runTests testCrateFlags testInputs testPreRun testPostRun; };

  /* Returns an attr set with packageId mapped to the result of buildRustCrateForPkgsFunc
    for the corresponding crate.
  */
  builtRustCratesWithFeatures =
    { packageId
    , features
    , crateConfigs ? crates
    , buildRustCrateForPkgsFunc
    , runTests
    , target ? defaultTarget
    } @ args:
      assert (builtins.isAttrs crateConfigs);
      assert (builtins.isString packageId);
      assert (builtins.isList features);
      assert (builtins.isAttrs target);
      assert (builtins.isBool runTests);
      let
        rootPackageId = packageId;
        mergedFeatures = mergePackageFeatures
          (
            args // {
              inherit rootPackageId;
              target = target // { test = runTests; };
            }
          );
        # Memoize built packages so that reappearing packages are only built once.
        builtByPackageIdByPkgs = mkBuiltByPackageIdByPkgs pkgs;
        mkBuiltByPackageIdByPkgs = pkgs:
          let
            self = {
              crates = lib.mapAttrs (packageId: value: buildByPackageIdForPkgsImpl self pkgs packageId) crateConfigs;
              build = mkBuiltByPackageIdByPkgs pkgs.buildPackages;
            };
          in
          self;
        buildByPackageIdForPkgsImpl = self: pkgs: packageId:
          let
            features = mergedFeatures."${packageId}" or [ ];
            crateConfig' = crateConfigs."${packageId}";
            crateConfig =
              builtins.removeAttrs crateConfig' [ "resolvedDefaultFeatures" "devDependencies" ];
            devDependencies =
              lib.optionals
                (runTests && packageId == rootPackageId)
                (crateConfig'.devDependencies or [ ]);
            dependencies =
              dependencyDerivations {
                inherit features target;
                buildByPackageId = depPackageId:
                  # proc_macro crates must be compiled for the build architecture
                  if crateConfigs.${depPackageId}.procMacro or false
                  then self.build.crates.${depPackageId}
                  else self.crates.${depPackageId};
                dependencies =
                  (crateConfig.dependencies or [ ])
                  ++ devDependencies;
              };
            buildDependencies =
              dependencyDerivations {
                inherit features target;
                buildByPackageId = depPackageId:
                  self.build.crates.${depPackageId};
                dependencies = crateConfig.buildDependencies or [ ];
              };
            filterEnabledDependenciesForThis = dependencies: filterEnabledDependencies {
              inherit dependencies features target;
            };
            dependenciesWithRenames =
              lib.filter (d: d ? "rename")
                (
                  filterEnabledDependenciesForThis
                    (
                      (crateConfig.buildDependencies or [ ])
                      ++ (crateConfig.dependencies or [ ])
                      ++ devDependencies
                    )
                );
            # Crate renames have the form:
            #
            # {
            #    crate_name = [
            #       { version = "1.2.3"; rename = "crate_name01"; }
            #    ];
            #    # ...
            # }
            crateRenames =
              let
                grouped =
                  lib.groupBy
                    (dependency: dependency.name)
                    dependenciesWithRenames;
                versionAndRename = dep:
                  let
                    package = crateConfigs."${dep.packageId}";
                  in
                  { inherit (dep) rename; version = package.version; };
              in
              lib.mapAttrs (name: choices: builtins.map versionAndRename choices) grouped;
          in
          buildRustCrateForPkgsFunc pkgs
            (
              crateConfig // {
                src = crateConfig.src or (
                  pkgs.fetchurl rec {
                    name = "${crateConfig.crateName}-${crateConfig.version}.tar.gz";
                    # https://www.pietroalbini.org/blog/downloading-crates-io/
                    # Not rate-limited, CDN URL.
                    url = "https://static.crates.io/crates/${crateConfig.crateName}/${crateConfig.crateName}-${crateConfig.version}.crate";
                    sha256 =
                      assert (lib.assertMsg (crateConfig ? sha256) "Missing sha256 for ${name}");
                      crateConfig.sha256;
                  }
                );
                extraRustcOpts = lib.lists.optional (targetFeatures != [ ]) "-C target-feature=${lib.concatMapStringsSep "," (x: "+${x}") targetFeatures}";
                inherit features dependencies buildDependencies crateRenames release;
              }
            );
      in
      builtByPackageIdByPkgs;

  /* Returns the actual derivations for the given dependencies. */
  dependencyDerivations =
    { buildByPackageId
    , features
    , dependencies
    , target
    }:
      assert (builtins.isList features);
      assert (builtins.isList dependencies);
      assert (builtins.isAttrs target);
      let
        enabledDependencies = filterEnabledDependencies {
          inherit dependencies features target;
        };
        depDerivation = dependency: buildByPackageId dependency.packageId;
      in
      map depDerivation enabledDependencies;

  /* Returns a sanitized version of val with all values substituted that cannot
    be serialized as JSON.
  */
  sanitizeForJson = val:
    if builtins.isAttrs val
    then lib.mapAttrs (n: v: sanitizeForJson v) val
    else if builtins.isList val
    then builtins.map sanitizeForJson val
    else if builtins.isFunction val
    then "function"
    else val;

  /* Returns various tools to debug a crate. */
  debugCrate = { packageId, target ? defaultTarget }:
    assert (builtins.isString packageId);
    let
      debug = rec {
        # The built tree as passed to buildRustCrate.
        buildTree = buildRustCrateWithFeatures {
          buildRustCrateForPkgsFunc = _: lib.id;
          inherit packageId;
        };
        sanitizedBuildTree = sanitizeForJson buildTree;
        dependencyTree = sanitizeForJson
          (
            buildRustCrateWithFeatures {
              buildRustCrateForPkgsFunc = _: crate: {
                "01_crateName" = crate.crateName or false;
                "02_features" = crate.features or [ ];
                "03_dependencies" = crate.dependencies or [ ];
              };
              inherit packageId;
            }
          );
        mergedPackageFeatures = mergePackageFeatures {
          features = rootFeatures;
          inherit packageId target;
        };
        diffedDefaultPackageFeatures = diffDefaultPackageFeatures {
          inherit packageId target;
        };
      };
    in
    { internal = debug; };

  /* Returns differences between cargo default features and crate2nix default
    features.

    This is useful for verifying the feature resolution in crate2nix.
  */
  diffDefaultPackageFeatures =
    { crateConfigs ? crates
    , packageId
    , target
    }:
      assert (builtins.isAttrs crateConfigs);
      let
        prefixValues = prefix: lib.mapAttrs (n: v: { "${prefix}" = v; });
        mergedFeatures =
          prefixValues
            "crate2nix"
            (mergePackageFeatures { inherit crateConfigs packageId target; features = [ "default" ]; });
        configs = prefixValues "cargo" crateConfigs;
        combined = lib.foldAttrs (a: b: a // b) { } [ mergedFeatures configs ];
        onlyInCargo =
          builtins.attrNames
            (lib.filterAttrs (n: v: !(v ? "crate2nix") && (v ? "cargo")) combined);
        onlyInCrate2Nix =
          builtins.attrNames
            (lib.filterAttrs (n: v: (v ? "crate2nix") && !(v ? "cargo")) combined);
        differentFeatures = lib.filterAttrs
          (
            n: v:
              (v ? "crate2nix")
              && (v ? "cargo")
              && (v.crate2nix.features or [ ]) != (v."cargo".resolved_default_features or [ ])
          )
          combined;
      in
      builtins.toJSON {
        inherit onlyInCargo onlyInCrate2Nix differentFeatures;
      };

  /* Returns an attrset mapping packageId to the list of enabled features.

    If multiple paths to a dependency enable different features, the
    corresponding feature sets are merged. Features in rust are additive.
  */
  mergePackageFeatures =
    { crateConfigs ? crates
    , packageId
    , rootPackageId ? packageId
    , features ? rootFeatures
    , dependencyPath ? [ crates.${packageId}.crateName ]
    , featuresByPackageId ? { }
    , target
      # Adds devDependencies to the crate with rootPackageId.
    , runTests ? false
    , ...
    } @ args:
      assert (builtins.isAttrs crateConfigs);
      assert (builtins.isString packageId);
      assert (builtins.isString rootPackageId);
      assert (builtins.isList features);
      assert (builtins.isList dependencyPath);
      assert (builtins.isAttrs featuresByPackageId);
      assert (builtins.isAttrs target);
      assert (builtins.isBool runTests);
      let
        crateConfig = crateConfigs."${packageId}" or (builtins.throw "Package not found: ${packageId}");
        expandedFeatures = expandFeatures (crateConfig.features or { }) features;
        enabledFeatures = enableFeatures (crateConfig.dependencies or [ ]) expandedFeatures;
        depWithResolvedFeatures = dependency:
          let
            packageId = dependency.packageId;
            features = dependencyFeatures enabledFeatures dependency;
          in
          { inherit packageId features; };
        resolveDependencies = cache: path: dependencies:
          assert (builtins.isAttrs cache);
          assert (builtins.isList dependencies);
          let
            enabledDependencies = filterEnabledDependencies {
              inherit dependencies target;
              features = enabledFeatures;
            };
            directDependencies = map depWithResolvedFeatures enabledDependencies;
            foldOverCache = op: lib.foldl op cache directDependencies;
          in
          foldOverCache
            (
              cache: { packageId, features }:
                let
                  cacheFeatures = cache.${packageId} or [ ];
                  combinedFeatures = sortedUnique (cacheFeatures ++ features);
                in
                if cache ? ${packageId} && cache.${packageId} == combinedFeatures
                then cache
                else
                  mergePackageFeatures {
                    features = combinedFeatures;
                    featuresByPackageId = cache;
                    inherit crateConfigs packageId target runTests rootPackageId;
                  }
            );
        cacheWithSelf =
          let
            cacheFeatures = featuresByPackageId.${packageId} or [ ];
            combinedFeatures = sortedUnique (cacheFeatures ++ enabledFeatures);
          in
          featuresByPackageId // {
            "${packageId}" = combinedFeatures;
          };
        cacheWithDependencies =
          resolveDependencies cacheWithSelf "dep"
            (
              crateConfig.dependencies or [ ]
              ++ lib.optionals
                (runTests && packageId == rootPackageId)
                (crateConfig.devDependencies or [ ])
            );
        cacheWithAll =
          resolveDependencies
            cacheWithDependencies "build"
            (crateConfig.buildDependencies or [ ]);
      in
      cacheWithAll;

  /* Returns the enabled dependencies given the enabled features. */
  filterEnabledDependencies = { dependencies, features, target }:
    assert (builtins.isList dependencies);
    assert (builtins.isList features);
    assert (builtins.isAttrs target);

    lib.filter
      (
        dep:
        let
          targetFunc = dep.target or (features: true);
        in
        targetFunc { inherit features target; }
        && (
          !(dep.optional or false)
          || builtins.any (doesFeatureEnableDependency dep) features
        )
      )
      dependencies;

  /* Returns whether the given feature should enable the given dependency. */
  doesFeatureEnableDependency = { name, rename ? null, ... }: feature:
    let
      prefix = "${name}/";
      len = builtins.stringLength prefix;
      startsWithPrefix = builtins.substring 0 len feature == prefix;
    in
    (rename == null && feature == name)
    || (rename != null && rename == feature)
    || startsWithPrefix;

  /* Returns the expanded features for the given inputFeatures by applying the
    rules in featureMap.

    featureMap is an attribute set which maps feature names to lists of further
    feature names to enable in case this feature is selected.
  */
  expandFeatures = featureMap: inputFeatures:
    assert (builtins.isAttrs featureMap);
    assert (builtins.isList inputFeatures);
    let
      expandFeature = feature:
        assert (builtins.isString feature);
        [ feature ] ++ (expandFeatures featureMap (featureMap."${feature}" or [ ]));
      outFeatures = lib.concatMap expandFeature inputFeatures;
    in
    sortedUnique outFeatures;

  /* This function adds optional dependencies as features if they are enabled
    indirectly by dependency features. This function mimics Cargo's behavior
    described in a note at:
    https://doc.rust-lang.org/nightly/cargo/reference/features.html#dependency-features
  */
  enableFeatures = dependencies: features:
    assert (builtins.isList features);
    assert (builtins.isList dependencies);
    let
      additionalFeatures = lib.concatMap
        (
          dependency:
            assert (builtins.isAttrs dependency);
            let
              enabled = builtins.any (doesFeatureEnableDependency dependency) features;
            in
            if (dependency.optional or false) && enabled then [ dependency.name ] else [ ]
        )
        dependencies;
    in
    sortedUnique (features ++ additionalFeatures);

  /*
    Returns the actual features for the given dependency.

    features: The features of the crate that refers this dependency.
  */
  dependencyFeatures = features: dependency:
    assert (builtins.isList features);
    assert (builtins.isAttrs dependency);
    let
      defaultOrNil =
        if dependency.usesDefaultFeatures or true
        then [ "default" ]
        else [ ];
      explicitFeatures = dependency.features or [ ];
      additionalDependencyFeatures =
        let
          dependencyPrefix = (dependency.rename or dependency.name) + "/";
          dependencyFeatures =
            builtins.filter (f: lib.hasPrefix dependencyPrefix f) features;
        in
        builtins.map (lib.removePrefix dependencyPrefix) dependencyFeatures;
    in
    defaultOrNil ++ explicitFeatures ++ additionalDependencyFeatures;

  /* Sorts and removes duplicates from a list of strings. */
  sortedUnique = features:
    assert (builtins.isList features);
    assert (builtins.all builtins.isString features);
    let
      outFeaturesSet = lib.foldl (set: feature: set // { "${feature}" = 1; }) { } features;
      outFeaturesUnique = builtins.attrNames outFeaturesSet;
    in
    builtins.sort (a: b: a < b) outFeaturesUnique;

  deprecationWarning = message: value:
    if strictDeprecation
    then builtins.throw "strictDeprecation enabled, aborting: ${message}"
    else builtins.trace message value;

  #
  # crate2nix/default.nix (excerpt end)
  #
  };
}

