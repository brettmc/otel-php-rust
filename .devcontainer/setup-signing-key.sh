#!/usr/bin/env bash
#
# Writes the public half of your SSH signing key into the container.
#
# VS Code forwards your SSH agent and copies your ~/.gitconfig into the
# container, but it never copies key files. So if you sign commits with an SSH
# key, your config arrives pointing user.signingkey at a path that does not
# exist here, and every commit fails with:
#
#   error: Couldn't load public key /home/php-rust/.ssh/id_ed25519.pub:
#   No such file or directory
#
# git needs the public key on disk to build the signature; the private half
# stays in the agent on your host and is never written here. This script
# reconstructs that one file from the forwarded agent on container start.
#
# It is a no-op unless you actually sign commits, so contributors who do not
# sign never see anything from it.

set -uo pipefail

# Container start must not fail over commit signing: someone who does not sign
# should see nothing, and a signer is better served by a message than by a
# container that will not open. Every exit below is therefore 0.
note() { echo "signing key: $*" >&2; }

# --- does this developer sign at all? ------------------------------------

# Only SSH-format signing needs a key file. gpg/x509 signing uses its own
# agent and keyring, which this script has no business touching.
[[ "$(git config --get gpg.format 2>/dev/null)" == "ssh" ]] || exit 0

# --type=bool normalises the several spellings git accepts for true.
[[ "$(git config --get --type=bool commit.gpgsign 2>/dev/null)" == "true" ]] || exit 0

signing_key="$(git config --get user.signingkey 2>/dev/null)"
[[ -n "${signing_key}" ]] || exit 0

# A literal key is handed straight to the agent, so no file is involved.
[[ "${signing_key}" == key::* ]] && exit 0

# --- where does git expect the file? -------------------------------------

# git expands a leading ~ itself, so a config value like ~/.ssh/id_ed25519.pub
# has to be expanded here too or we would create a literal "~" directory.
target="${signing_key/#\~/${HOME}}"

# user.signingkey may name either half of the pair. ssh-keygen signs with
# `-f <private>` and reads `<private>.pub`, so the public half belongs at
# <value>.pub unless the value already points at it.
[[ "${target}" == *.pub ]] || target="${target}.pub"

# Never overwrite a key that is already here: it may be the real one, mounted
# or written by hand, and clobbering it could silently change signing identity.
[[ -e "${target}" ]] && exit 0

# --- find the agent ------------------------------------------------------

# Lifecycle hooks do not reliably inherit SSH_AUTH_SOCK from the user session,
# so fall back to the sockets VS Code binds into /tmp. Reconnecting leaves
# stale ones behind, so try each and keep the first that answers.
if ! ssh-add -l >/dev/null 2>&1; then
  shopt -s nullglob
  for sock in /tmp/vscode-ssh-auth-*.sock; do
    SSH_AUTH_SOCK="${sock}"
    export SSH_AUTH_SOCK
    ssh-add -l >/dev/null 2>&1 && break
  done
  shopt -u nullglob
fi

# ssh-add -L prints public keys; -l prints fingerprints. We need the former.
# Exit status alone is not enough: an agent with no keys succeeds but is empty.
if ! agent_keys="$(ssh-add -L 2>/dev/null)" || [[ -z "${agent_keys}" ]] \
  || [[ "${agent_keys}" == *"has no identities"* ]]; then
  note "no key found in a forwarded SSH agent, so ${target} was not written."
  note "commits will fail to sign. check that your agent is running and holds"
  note "your key on the host (ssh-add -L), then restart the container."
  exit 0
fi

# --- pick the right key --------------------------------------------------

if [[ "$(grep -c . <<<"${agent_keys}")" -eq 1 ]]; then
  key="${agent_keys}"
else
  # Signing with the wrong key is worse than not signing: git produces a
  # signature happily, and the commit then shows as unverified on GitHub,
  # which is an obscure thing to trace back to this script. So match the key
  # comment against user.email and write nothing when that is ambiguous.
  email="$(git config --get user.email 2>/dev/null)"
  key=""
  if [[ -n "${email}" ]]; then
    key="$(grep -F -- " ${email}" <<<"${agent_keys}" | head -1)"
  fi
  if [[ -z "${key}" ]]; then
    note "your agent holds several keys and none is commented <${email:-unset}>,"
    note "so none was chosen rather than risk signing with the wrong one."
    note "write the right one yourself, then restart the container:"
    note "  ssh-add -L | grep <your-key-comment> > ${target}"
    exit 0
  fi
fi

# --- write it ------------------------------------------------------------

# ssh refuses to use a key directory group- or world-writable.
if ! mkdir -p "$(dirname "${target}")" || ! chmod 700 "$(dirname "${target}")"; then
  note "could not create $(dirname "${target}")."
  exit 0
fi

# Write through a temporary file in the same directory so an interrupted run
# cannot leave a truncated key where git will try to read one.
tmp="$(mktemp "${target}.XXXXXX")" || { note "could not write ${target}."; exit 0; }
if printf '%s\n' "${key}" >"${tmp}" && chmod 644 "${tmp}" && mv -f "${tmp}" "${target}"; then
  note "wrote ${target} (${key##* }) from the forwarded agent."
else
  rm -f "${tmp}"
  note "could not write ${target}."
fi

exit 0
