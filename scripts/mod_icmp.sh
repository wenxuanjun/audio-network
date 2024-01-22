#!/bin/sh

current_value=$(cat /proc/sys/net/ipv4/icmp_echo_ignore_all)

if [ "$current_value" -eq 0 ]; then
    echo 1 > /proc/sys/net/ipv4/icmp_echo_ignore_all
    echo "Disabling ICMP ping response!"
elif [ "$current_value" -eq 1 ]; then
    echo 0 > /proc/sys/net/ipv4/icmp_echo_ignore_all
    echo "Enabling ICMP ping response!"
else
    echo "Unexpected value: $current_value"
fi