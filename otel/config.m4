PHP_ARG_ENABLE(otel, whether to enable otel, [ --enable-otel Enable otel])

if test "$PHP_OTEL" != "no"; then
  AC_PATH_PROG([CARGO], [cargo])
  if test -z "$CARGO"; then
    AC_MSG_ERROR([Cargo not found! Install Rust via https://rustup.rs])
  fi

  AC_PATH_PROG([RUSTC], [rustc])
  if test -z "$RUSTC"; then
    AC_MSG_ERROR([Rust compiler (rustc) not found! Install Rust via https://rustup.rs])
  fi

  RUSTC_VERSION=$($RUSTC --version | cut -d' ' -f2)
  echo "rustc === ::${RUSTC_VERSION}::"
  RUSTC_REQUIRED_VERSION="1.85.0"
  AC_MSG_CHECKING([Rust version found: $RUSTC_VERSION])

  # Compare versions using Autoconf's built-in version comparison
  AS_VERSION_COMPARE([$RUSTC_REQUIRED_VERSION], [$RUSTC_VERSION],
    [AC_MSG_RESULT([OK])]
    , []
    , [AC_MSG_ERROR([rustc >= $RUSTC_REQUIRED_VERSION is required])]
  )

  # Fake extension configuration
  PHP_NEW_EXTENSION(otel, otel.c, $ext_shared)

  AC_MSG_CHECKING([for existing Makefile])
  if test -f Makefile; then
    AC_MSG_RESULT([found])
    AC_MSG_NOTICE([Backing up existing Makefile...])
    mv Makefile Makefile.bak
  else
    AC_MSG_RESULT([not found])
  fi

  # After ./configure runs, restore the original Makefile
    AC_CONFIG_COMMANDS_POST([
      if test -f Makefile.bak; then
        AC_MSG_NOTICE([Restoring original Makefile...])
        mv Makefile.bak Makefile
      fi
    ])
fi
