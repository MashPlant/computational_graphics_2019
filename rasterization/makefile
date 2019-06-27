.PHONY: all run clean

all: main

main: main.cpp pic.h lodepng.o
	g++ -fopenmp -O3 -std=c++17 main.cpp lodepng.o -o main

lodepng.o: lodepng.cpp lodepng.h
	g++ -O3 -c lodepng.cpp -o lodepng.o

run: all
	./main

clean:
	rm main *.o