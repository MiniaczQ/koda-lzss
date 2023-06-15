#!/bin/bash

for ((i=4;i<=16;i++));
do 
   for ((j=4;j<=$i;j++));
    do 
        ./test_on_data.sh $i $j
    done
done
