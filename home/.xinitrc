#!/usr/bin/env bash

if [ -d /etc/X11/xinit/xinitrc.d ] ; then
  for f in /etc/X11/xinit/xinitrc.d/?*.sh ; do
    [ -x "$f" ] && . "$f"
  done
  unset f
fi

[ -f ~/.Xresources ] && xrdb ~/.Xresources
[ -f ~/.xprofile ] && . ~/.xprofile

exec i3
