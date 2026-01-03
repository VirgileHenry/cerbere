{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    llvmPackages.llvm
    llvmPackages.clang
    llvmPackages.libclang
    pam
  ];

  LLVM_CONFIG_PATH = "${pkgs.llvmPackages.llvm}/bin/llvm-config";
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
