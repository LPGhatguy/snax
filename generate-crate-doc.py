# Generates snax/src/lib.rs based on README.md

import re

with open("README.md", "r") as file:
	# Read the first line into oblivion, which contains the README title
	file.readline()
	readmeContents = file.read()

with open("snax/src/lib.rs", "r") as file:
	libContents = file.read()

existingDocPattern = re.compile("(//![^\n]*\n)+", re.MULTILINE)
linePattern = re.compile("(.*)")

existingDocMatch = existingDocPattern.match(libContents)
libBody = libContents[existingDocMatch.end():]
docComment = linePattern.sub("//! \\1", readmeContents) + "\n"

newLibContents = docComment + libBody

with open("snax/src/lib.rs", "w") as file:
	file.write(newLibContents)