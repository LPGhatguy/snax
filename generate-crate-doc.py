# Generates crate-level documentation from README.md

import re

readmePath = "README.md"
libPath = "src/lib.rs"

with open(readmePath, "r") as file:
	# Read the first line into oblivion, which contains the title
	file.readline()
	readmeContents = file.read()

with open(libPath, "r") as file:
	libContents = file.read()

existingDocPattern = re.compile("^(//![^\n]*\n)+", re.MULTILINE)

existingDocMatch = existingDocPattern.match(libContents)
code = libContents[existingDocMatch.end():]

docLines = []
for line in readmeContents.splitlines():
	newLine = ("//! " + line).strip()
	docLines.append(newLine)

docComment = "\n".join(docLines) + "\n"

newLibContents = docComment + code

with open(libPath, "w") as file:
	file.write(newLibContents)