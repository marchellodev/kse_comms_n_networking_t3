package main

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"math/rand"
	"net"
	"os"
	"time"
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
			write(writer, []uint32{5})
			fmt.Println("> ping sent")

		case "send_4":
			matrix1 := generateMatrix(4)
			matrix2 := generateMatrix(4)
			fmt.Println("> matrices generated")

			write(writer, []uint32{7, 4})
			for _, row := range matrix1 {
				write(writer, row)
			}
			for _, row := range matrix2 {
				write(writer, row)
			}

			fmt.Println("> matrices sent")

		default:
			fmt.Println("Invalid command.")
		}
	}
}

func generateMatrix(size int) [][]uint32 {
	matrix := make([][]uint32, size)
	rand.Seed(time.Now().UnixNano())

	for i := range matrix {
		matrix[i] = make([]uint32, size)
		for j := range matrix[i] {
			matrix[i][j] = uint32(rand.Intn(1000)) // Generate a random value from 1 to 1000
		}
	}

	return matrix
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

			case 8:
				id := read(reader)

				fmt.Println("> matrix pair stored with id:", id)

			default:
				fmt.Println("> unknown received:", number)
			}
		}
	}()
}

func read(reader *bufio.Reader) uint32 {
	buffer := make([]byte, 4)
	_, err := reader.Read(buffer)
	number := binary.BigEndian.Uint32(buffer)
	if err != nil {
		fmt.Println("Failed to read server response:", err.Error())
		os.Exit(1)
	}
	return number;
}

// expects a row of numbers, automatically transforms them to bytes
func write(writer *bufio.Writer, numbers []uint32) {
	numBytes := make([]byte, 4)
	for _, number := range numbers {
		// fmt.Println("> writing:", number)
		binary.BigEndian.PutUint32(numBytes, number)
		_, err := writer.Write([]byte(numBytes))
		if err != nil {
			println("Write failed:", err.Error())
			os.Exit(1)
		}
	}
	err := writer.Flush()
	if err != nil {
		println("Flush failed:", err.Error())
		os.Exit(1)
	}
}
