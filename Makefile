%.o : %.S
	i686-elf-as $< -o $@

all : boot.o

clean :
	rm -rf *.o
	rm -rf *.bin
	