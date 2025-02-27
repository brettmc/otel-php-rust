PHP_ARG_ENABLE(otel, whether to enable otel, [ --enable-otel Enable otel])

if test "$PHP_OTEL" != "no"; then
  AC_MSG_CHECKING([for Rust build system])
  AC_MSG_RESULT([using Cargo instead of Autotools])

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
