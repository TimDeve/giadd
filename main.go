package main

import (
	"bytes"
	"fmt"
	"log"
	"os/exec"
	"strings"
)

type file struct {
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

	fmt.Printf("%q\n", files)
}

func marshallOutputInFiles(outputStr string) []file {
	var files []file

	for _, str := range strings.Split(outputStr, "\n") {
		str = standardizeSpaces(str)
		splitedStr := strings.Split(str, " ")
		if len(splitedStr) == 2 {
			thisFile := file{
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
