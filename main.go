package main

import (
	"bytes"
	"fmt"
	"log"
	"os/exec"
	"strings"
	"unicode/utf8"

	"github.com/timdeve/giadd/screen"
)

type File struct {
	Status string
	File   string
}

func main() {
	cmd := exec.Command("git", "status", "-s")
	var out bytes.Buffer
	cmd.Stdout = &out
	err := cmd.Run()
	if err != nil {
		log.Fatal(err)
	}

	files := marshallOutputInFiles(out.String())

	printList(files)
}

func printList(files []File) {
	longest := findLongestStatus(files)
	var lines []string

	for i, file := range files {
		line := fmt.Sprintf("%v) %v %v", i+1, padRight(file.Status, longest), file.File)
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
				Status: splitedStr[0],
				File:   splitedStr[1],
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
