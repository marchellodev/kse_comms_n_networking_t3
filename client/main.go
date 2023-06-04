package main

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"net"
	"os"
)

func main() {
	tcpServer, err := net.ResolveTCPAddr("tcp", "localhost:7878")

	conn, err := net.DialTCP("tcp", nil, tcpServer)
	if err != nil {
		println("Dial failed:", err.Error())
		os.Exit(1)
	}
	writer := bufio.NewWriter(conn)
	reader := bufio.NewReader(conn)

	initReader(reader)

	executeCLI(writer)
}

func executeCLI(writer *bufio.Writer) {
	fmt.Println("Connection established.")

	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		command := scanner.Text()
		switch command {
		case "ping":
			write(writer, 5)
			fmt.Println("> ping sent")
		default:
			fmt.Println("Invalid command.")
		}
	}
}

func initReader(reader *bufio.Reader) {
	go func() {
		for {
			buffer := make([]byte, 4)
			_, err := reader.Read(buffer)
			number := binary.BigEndian.Uint32(buffer)
			if err != nil {
				fmt.Println("Failed to read server response:", err.Error())
				os.Exit(1)
			}
			switch number {
			case 6:
				fmt.Println("> pong received")
			default:
				fmt.Println("> unknown received:", number)
			}
		}
	}()
}

func write(writer *bufio.Writer, number uint32) {
	numBytes := make([]byte, 4)
	binary.BigEndian.PutUint32(numBytes, number)
	_, err := writer.Write([]byte(numBytes))
	if err != nil {
		println("Write failed:", err.Error())
		os.Exit(1)
	}
	err = writer.Flush()
}
