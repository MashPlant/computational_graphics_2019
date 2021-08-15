#!/bin/bash
for i in {1..6}
do
  expect -c "spawn scp smallpt.cpp st${i}@123.57.237.171:~/
             expect \"*password:\" { send \"12345678\n\" }
             expect eof"
  expect -c "spawn ssh st${i}@123.57.237.171
             expect \"*password:\" { send \"12345678\n\" }
             expect st${i}* { exit }"
done

# spawn scp smallpt.cpp st${i}@123.57.237.171:~/
# expect "*password:"
# send "12345678\n"
# interact

# st10:x:1067:
# st1ddd:x:1078:
# sx10:x:1088:
# st2suo:x:1092:
# st1:x:1093:
# st2:x:1094:
# st3:x:1095:
# st4:x:1096:
# st5:x:1097:
# st6:x:1098: