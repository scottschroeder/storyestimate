# -*- mode: ruby -*-
# vi: set ft=ruby :

VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|

    #
    # Set the default vm boxes
    #

    config.vm.provider "virtualbox" do |v|
        # toggle gui on as needed
        v.gui = false

        # beef up the vm memory to help with large builds
        v.memory = '512'

        # time sync fiddling
        v.customize [ "guestproperty", "set", :id, "/VirtualBox/GuestAdd/VBoxService/--timesync-set-threshold", 10000 ]

        # dns fiddling
        v.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
        v.customize ["modifyvm", :id, "--natdnsproxy1", "on"]
    end

    #
    # Setup self-contained build box
    #

    config.vm.define "build_box", primary: true, autostart: true do |the_box|
        #
        # Box base image
        #
        the_box.vm.box = "ubuntu/xenial64"

        #
        # Do vm setup
        #
        the_box.vm.hostname = "se-build-box"
        the_box.vm.network "private_network", ip: "10.10.0.28"

        #
        # Sync src folder
        #
        the_box.vm.synced_folder "./", "/vagrant"

        #
        # Setup the build vm
        #
        the_box.vm.provision "shell", privileged: true, inline: "apt-get update --fix-missing"
        the_box.vm.provision "shell", privileged: true, inline: "apt-get install -q -y ruby-dev build-essential git tree g++ make curl vim"
        the_box.vm.provision "shell", privileged: true, inline: "gem install fpm"

        # Rust Setup
        the_box.vm.provision "shell", inline: "curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly"

        #
        # Run the build
        #
        the_box.vm.provision "shell", privileged: true, inline: "cp -r /vagrant /root/storyestimates"
        the_box.vm.provision "shell", privileged: true, inline: "cd /root/storyestimates && make clean && make all"

        #
        # Test if the build installs properly
        #
        the_box.vm.provision "shell", privileged: true, inline: "dpkg --force-all -i /root/storyestimates/build/story-estimates-webapp_*_all.deb"
        the_box.vm.provision "shell", privileged: true, inline: "test -f /opt/storyestimates/bin/estimate  && echo 'PASS: Deb install looks good!' || echo ERROR: Expected file was not installed properly. >&2"
        the_box.vm.provision "shell", privileged: true, inline: "cp -r /root/storyestimates/build/* /vagrant/build/"

    end

    config.vm.define "test_box", primary: true, autostart: true do |the_box|
        #
        # Box base image
        #
        the_box.vm.box = "ubuntu/xenial64"

        #
        # Do vm setup
        #
        the_box.vm.hostname = "se-systest"
        the_box.vm.network "private_network", ip: "10.10.0.29"

        the_box.vm.network "forwarded_port", guest: 8080, host: 8080


        #
        # Sync src folder
        #
        the_box.vm.synced_folder "./", "/vagrant"

        #
        # Setup the build vm
        #
        the_box.vm.provision "shell", privileged: true, inline: "apt-get update --fix-missing"
        the_box.vm.provision "shell", privileged: true, inline: "apt-get install -q -y git tree curl vim"

        #
        # Install app requirements
        #
        the_box.vm.provision "shell", privileged: true, inline: "apt-get install -q -y redis-server nginx"


        #
        # Test if the build installs properly
        #
        the_box.vm.provision "shell", privileged: true, inline: "dpkg --force-all -i /vagrant/build/story-estimates-webapp_*_all.deb"
        the_box.vm.provision "shell", privileged: true, inline: "test -f /opt/storyestimates/bin/estimate  && echo 'PASS: Deb install looks good!' || echo ERROR: Expected file was not installed properly. >&2"


        #
        # Setup the box for testing
        #
        the_box.vm.provision "shell", privileged: true, inline: "rm -v /etc/nginx/sites-enabled/default"
        the_box.vm.provision "shell", privileged: true, inline: "ln -s /opt/storyestimates/config/nginx/storyestimates.conf /etc/nginx/sites-enabled/"

    end
end
