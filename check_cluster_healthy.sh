#!/bin/bash

echo "Checking if ScyllaDB cluster is healthy..."

while true; do
   (echo >/dev/tcp/scylladb-01/9042) &>/dev/null 
   if [ "$?" == "0" ]; then
      echo "scylladb-01 is ready" 
      SCYLLA1_READY="true"
   else
      echo "Waiting for scylladb-01 CQL port"
      SCYLLA1_READY="false"
   fi

   (echo >/dev/tcp/scylladb-02/9042) &>/dev/null 
   if [ "$?" == "0" ]; then
      echo "scylladb-02 is ready"
      SCYLLA2_READY="true"
   else
      echo "Waiting for scylladb-02 CQL port"
      SCYLLA2_READY="false"
   fi

   SCYLLA3_READY="true"

   # (echo >/dev/tcp/scylladb-03/9042) &>/dev/null 
   # if [ "$?" == "0" ]; then
   #    echo "scylladb-03 is ready"
   #    SCYLLA3_READY="true"
   # else
   #    echo "Waiting for scylladb-03 CQL port"
   #    SCYLLA3_READY="false"
   # fi

   if [ "$SCYLLA1_READY" == "$SCYLLA2_READY" ]; then
      if [ "$SCYLLA2_READY" == "$SCYLLA3_READY" ]; then
         if [ "$SCYLLA1_READY" == "true" ]; then
            echo "ScyllaDB is up and available. You are ready to go!"
            break
         fi
      fi
   fi
   sleep 5
done

sleep 10

./message-service