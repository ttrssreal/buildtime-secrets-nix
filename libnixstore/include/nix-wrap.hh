#pragma once

#include <nix/store/store-api.hh>
#include <nix/store/store-open.hh>

#include "rust/cxx.h"

namespace wrap {
enum class NixErrorTag : std::uint8_t;
class StorePath;

class LocalStore {
public:
  // Requires libnixstore to be initialized
  LocalStore();

  rust::String get_version() const;
  std::shared_ptr<StorePath>
  parse_store_path(rust::Slice<const std::uint8_t> path) const;
  rust::String get_derivation_env_val(std::shared_ptr<StorePath> path,
                                      rust::Str key) const;
  rust::String get_derivation_name(std::shared_ptr<StorePath> path) const;
  rust::String get_store_relative_path(std::shared_ptr<StorePath> path) const;

private:
  std::shared_ptr<nix::Store> store;
};

class FfiError : public nix::BaseError {
public:
  template <typename... Args>
  explicit FfiError(NixErrorTag tag, const std::string &fs,
                    const Args &...args);

  explicit FfiError(NixErrorTag tag, const nix::BaseError base_error);

  const char *what() const noexcept override;

private:
  std::string msg;
};

class StorePath {
public:
  explicit StorePath(const nix::ValidPathInfo &path_info)
      : valid_path_info(std::move(path_info)) {}

  nix::ValidPathInfo valid_path_info;
};

std::unique_ptr<LocalStore> new_local_store();
void init_lib_nix_store();
} // namespace wrap
