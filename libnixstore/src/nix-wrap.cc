#include <nix/main/shared.hh>
#include <nix/store/globals.hh>
#include <nix/store/path.hh>

#include "libnixstore/include/nix-wrap.hh"
#include "libnixstore/src/lib.rs"
#include "rust/cxx.h"

namespace wrap {
void init_lib_nix_store() { nix::initLibStore(); }

std::shared_ptr<StorePath>
LocalStore::parse_store_path(rust::Slice<const std::uint8_t> path) const {
  auto path_string =
      std::string(reinterpret_cast<const char *>(path.data()), path.size());
  std::optional<nix::StorePath> store_path;
  std::optional<nix::ref<const nix::ValidPathInfo>> valid_path_info;

  try {
    // Check syntactic validity
    store_path = store->parseStorePath(path_string);
  } catch (const nix::BaseError &e) {
    throw FfiError(NixErrorTag::StorePath, e);
  }

  try {
    // Check semantic validity
    valid_path_info = store->queryPathInfo(store_path.value());
  } catch (const nix::BaseError &e) {
    throw FfiError(NixErrorTag::StorePath, e);
  }

  return std::make_shared<StorePath>(*(valid_path_info.value()));
}

rust::String LocalStore::get_derivation_env_val(std::shared_ptr<StorePath> path,
                                                rust::Str key) const {
  nix::Derivation derivation =
      store->readDerivation(path->valid_path_info.path);
  std::string key_str(key);

  auto it = derivation.env.find(key_str);

  if (it == derivation.env.end())
    throw FfiError(NixErrorTag::EnvKeyDoesNotExist,
                   "derivation environment value for key '%s' doesn't exist",
                   key_str);

  return it->second;
}

rust::String
LocalStore::get_store_relative_path(std::shared_ptr<StorePath> path) const {
  return std::string(path->valid_path_info.path.to_string());
}

rust::String
LocalStore::get_derivation_name(std::shared_ptr<StorePath> path) const {
  nix::Derivation derivation =
      store->readDerivation(path->valid_path_info.path);
  return derivation.name;
}

rust::String LocalStore::get_version() const {
  auto version = store->getVersion();

  if (!version)
    throw FfiError(NixErrorTag::GetVersion, "store returned no nix version");

  return version.value();
}

std::unique_ptr<LocalStore> new_local_store() {
  return std::unique_ptr<LocalStore>(new LocalStore());
}

FfiError::FfiError(NixErrorTag tag, const nix::BaseError base_error)
    : nix::BaseError(base_error) {
  auto base_msg = base_error.what();
  auto tag_str = std::to_string(static_cast<uint8_t>(tag));
  msg = std::format("{},{}", tag_str, base_msg);
}

template <typename... Args>
FfiError::FfiError(NixErrorTag tag, const std::string &fs, const Args &...args)
    : nix::BaseError(fs, args...) {
  auto base_error = calcWhat();
  auto tag_str = std::to_string(static_cast<uint8_t>(tag));
  msg = std::format("{},{}", tag_str, base_error);
}

const char *FfiError::what() const noexcept { return msg.c_str(); }

LocalStore::LocalStore() : store(nix::openStore()) {}
} // namespace wrap
