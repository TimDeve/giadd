package main

import (
	"bytes"
	"fmt"
	"log"
	"os/exec"
	"strings"
	"unicode/utf8"

	"github.com/pkg/term"
	"github.com/timdeve/giadd/screen"
)

type File struct {
	Status   string
	File     string
	Selected bool
}

var (
	currentCursorPosition int
	files                 []File
)

func main() {
	cmd := exec.Command("git", "status", "-s")
	var out bytes.Buffer
	cmd.Stdout = &out
	err := cmd.Run()
	if err != nil {
		log.Fatal(err)
	}

	files = marshallOutputInFiles(out.String())

	printList(files)
	readKeyPress()
}

func readKeyPress() {
keyPressLoop:
	for {
		ascii, _, _ := getChar()

		switch ascii {
		case 27:
			screen.Print([]string{"Exiting without applying."})
			break keyPressLoop
		case 113:
			screen.Print([]string{"Exiting without applying."})
			break keyPressLoop
		case 13:
			addMarkedFiles()
			screen.Print([]string{"Added Selected files."})
			break keyPressLoop
		case 32:
			markSelectedFile()
			printList(files)
		case 107:
			cursorPositionUp()
			printList(files)
		case 106:
			cursorPositionDown()
			printList(files)
		}
	}
}

func addMarkedFiles() {
	filesToAdd := []string{"add"}

	for _, file := range files {
		if file.Selected {
			filesToAdd = append(filesToAdd, file.File)
		}
	}

	cmd := exec.Command("git", filesToAdd...)
	var out bytes.Buffer
	cmd.Stdout = &out
	err := cmd.Run()
	if err != nil {
		log.Fatal(err)
	}
}

func cursorPositionUp() {
	currentCursorPosition--
	if currentCursorPosition < 0 {
		currentCursorPosition = len(files) - 1
	}
}

func cursorPositionDown() {
	currentCursorPosition++
	if currentCursorPosition > len(files)-1 {
		currentCursorPosition = 0
	}
}

func markSelectedFile() {
	files[currentCursorPosition].Selected = !files[currentCursorPosition].Selected
}

// Returns either an ascii code, or (if input is an arrow) a Javascript key code.
func getChar() (ascii int, keyCode int, err error) {
	t, _ := term.Open("/dev/tty")
	term.RawMode(t)
	bytes := make([]byte, 3)

	var numRead int
	numRead, err = t.Read(bytes)
	if err != nil {
		return
	}
	if numRead == 3 && bytes[0] == 27 && bytes[1] == 91 {
		// Three-character control sequence, beginning with "ESC-[".

		// Since there are no ASCII codes for arrow keys, we use
		// Javascript key codes.
		if bytes[2] == 65 {
			// Up
			keyCode = 38
		} else if bytes[2] == 66 {
			// Down
			keyCode = 40
		} else if bytes[2] == 67 {
			// Right
			keyCode = 39
		} else if bytes[2] == 68 {
			// Left
			keyCode = 37
		}
	} else if numRead == 1 {
		ascii = int(bytes[0])
	} else {
		// Two characters read??
	}
	t.Restore()
	t.Close()
	return
}

func printList(files []File) {
	longest := findLongestStatus(files)
	var lines []string
	var cursor string
	var selected string

	for i, file := range files {
		if i == currentCursorPosition {
			cursor = ">"
		} else {
			cursor = " "
		}

		if file.Selected {
			selected = "*"
		} else {
			selected = " "
		}

		line := fmt.Sprintf("%v [%v] %v %v", cursor, selected, padRight(file.Status, longest), file.File)
		lines = append(lines, line)
	}

	screen.Print(lines)
}

func marshallOutputInFiles(outputStr string) []File {
	var files []File

	for _, str := range strings.Split(outputStr, "\n") {
		str = standardizeSpaces(str)
		splitedStr := strings.Split(str, " ")
		if len(splitedStr) == 2 {
			thisFile := File{
				Status:   splitedStr[0],
				File:     splitedStr[1],
				Selected: false,
			}
			files = append(files, thisFile)
		}
	}

	return files
}

func standardizeSpaces(str string) string {
	return strings.Join(strings.Fields(str), " ")
}

func padRight(str string, length int) string {
	for {
		if len(str) >= length {
			return str
		}
		str = str + " "
	}
}

func findLongestStatus(files []File) int {
	var longest int
	for _, file := range files {
		length := utf8.RuneCountInString(file.Status)
		if length > longest {
			longest = length
		}
	}
	return longest
}
