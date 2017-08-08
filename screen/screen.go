package screen

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
)

type termSize struct {
	Width  int
	Height int
}

var maxLines int

// Print prints out an array of string to the screen by clearing it first
func Print(lines []string) {
	clearScreen()

	if maxLines < len(lines) {
		maxLines = len(lines)
	}

	for _, line := range lines {
		fmt.Println(line)
	}
}

func moveCursorUp(lineNumber int) {
	if maxLines > 0 {
		fmt.Printf(fmt.Sprintf("\033[%dA", lineNumber))
	}
}

func genCleanLine() string {
	term, _ := getTermSize()
	var buffer bytes.Buffer

	for i := 0; i < term.Width; i++ {
		buffer.WriteString(" ")
	}

	return buffer.String()
}

func clearScreen() {
	cleanLine := genCleanLine()

	moveCursorUp(maxLines)

	for i := 0; i < maxLines; i++ {
		fmt.Println(cleanLine)
	}

	moveCursorUp(maxLines)
}

func getTermSize() (termSize, error) {
	cmd := exec.Command("stty", "size")
	cmd.Stdin = os.Stdin
	out, err := cmd.Output()
	if err != nil {
		return termSize{}, err
	}
	outString := string(out)

	splitOut := strings.Split(outString, "\n")
	splitOut = strings.Split(splitOut[0], " ")

	height, err := strconv.Atoi(splitOut[0])
	if err != nil {
		return termSize{}, err
	}
	width, err := strconv.Atoi(splitOut[1])
	if err != nil {
		return termSize{}, err
	}

	term := termSize{Height: height, Width: width}
	return term, nil
}
