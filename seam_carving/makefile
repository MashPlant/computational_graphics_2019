seam_carving: main.cpp lodepng.o cmdline.o
	clang++ -Ofast -march=native -fopenmp main.cpp lodepng.o cmdline.o -DLAPLACE=1 -DNEED_SEAM=1 -o seam_carving

lodepng.o: lodepng/lodepng.cpp lodepng/lodepng.h
	clang++ -Ofast -march=native lodepng/lodepng.cpp -c -o lodepng.o

cmdline.o: cmdline/cmdline.cpp cmdline/cmdline.h
	clang++ -Ofast -march=native cmdline/cmdline.cpp -c -o cmdline.o

clean:
	rm seam_carving lodepng.o cmdline.o