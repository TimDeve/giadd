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

type file struct {
	Status   string
	File     string
	Selected bool
}

var (
	currentCursorPosition int
	files                 []file
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
	if len(files) > 0 {
		readKeyPress()
	} else {
		screen.Print([]string{"No modified files. Exiting."})
	}
}

func readKeyPress() {
keyPressLoop:
	for {
		ascii, _ := getChar()

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

	for _, f := range files {
		if f.Selected {
			filesToAdd = append(filesToAdd, f.File)
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

func getChar() (ascii int, err error) {
	t, _ := term.Open("/dev/tty")
	term.RawMode(t)
	bytes := make([]byte, 3)

	var numRead int
	numRead, err = t.Read(bytes)
	if err != nil {
		return
	}

	if numRead == 1 {
		ascii = int(bytes[0])
	}

	t.Restore()
	t.Close()
	return
}

func printList(files []file) {
	longest := findLongestStatus(files)
	var lines []string
	var cursor string
	var selected string

	for i, f := range files {
		if i == currentCursorPosition {
			cursor = ">"
		} else {
			cursor = " "
		}

		if f.Selected {
			selected = "*"
		} else {
			selected = " "
		}

		line := fmt.Sprintf("%v [%v] %v %v", cursor, selected, padRight(f.Status, longest), f.File)
		lines = append(lines, line)
	}

	screen.Print(lines)
}

func marshallOutputInFiles(outputStr string) []file {
	var files []file

	for _, str := range strings.Split(outputStr, "\n") {
		str = standardizeSpaces(str)
		splitedStr := strings.Split(str, " ")
		if len(splitedStr) == 2 {
			thisFile := file{
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

func findLongestStatus(files []file) int {
	var longest int
	for _, f := range files {
		length := utf8.RuneCountInString(f.Status)
		if length > longest {
			longest = length
		}
	}
	return longest
}
