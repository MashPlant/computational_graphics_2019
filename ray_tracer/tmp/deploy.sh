#!/bin/bash
cargo run
f=ray_tracer.cpp
# for i in {1..5}
# do
#   expect -c "spawn scp $f st${i}@123.57.237.171:~/
#              expect \"*password:\" { send \"12345678\r\" }
#              expect eof"
#   expect -c "set timeout -1
#              spawn ssh st${i}@123.57.237.171
#              expect \"*password:\" { send \"12345678\r\" }
#              expect st${i}* { send \"g++ -O3 -fopenmp -ffast-math -march=native -std=c++11 $f\r\" }
#              expect st${i}* { send \"./a.out 100 $i ${i}.raw >/dev/null 2>&1 &\r\" }
#              expect st${i}* { send \"sleep 0.5\r\" }
#              expect st${i}* { send \"exit\r\" }
#              expect st${i}* { send \"sleep 0.5\r\" }
#              "
# done
for i in {1..6}
do
  expect -c "set timeout -1
             spawn ssh st${i}@123.57.237.171
             expect \"*password:\" { send \"12345678\r\" }
             expect st${i}* { send \"rm $f\r\" }
             expect st${i}* { send \"rm smallpt.cpp\r\" }
             expect st${i}* { send \"rm a.out\r\" }
             expect st${i}* { send \"rm ${i}.raw\r\" }
             expect st${i}* { send \"sleep 0.5\r\" }
             expect st${i}* { send \"exit\r\" }
             expect st${i}* { send \"sleep 0.5\r\" }
             "
done

