# Autofix

Many of RSLint's rules can sometimes or always be automatically fixed, this is accomplished through the `--fix` (`-f`) flag.
In order to not cause more issues and potentially apply incorrect fixes, fixes are not applied if the file contains any syntax errors. To get around this
behavior, you can use the `--dirty` (`-D`) flag, use it at your own risk!

## Issues which can be automatically fixed

RSLint opts for a slightly more risky but very powerful policy when it comes to fixes. Fixes may potentially change program behavior if the behavior intended is
an error. For example, RSLint can automatically fix `new Symbol()` by deleting the `new`, this constitutes a change in incorrect behavior since the old behavior causes a TypeError 100% of the time. Fixes should however never change program behavior in a great way or change otherwise "correct" behavior.
