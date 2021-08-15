all:
	cargo run --release
	nvcc -O3 -use_fast_math ray_tracer.cu mesh0.o
	./a.out 1024