// A bunch of functions to make working with the dom easier

function p() {
    return new El('p');
}

function div() {
    return new El('div');
}

function button() {
    return new El('button');
}

function fieldset() {
    return new El('fieldset');
}

function label() {
    return new El('label');
}

function input() {
    let input = new El('input');
    input.type = function(type) {
        input.el.type = type;
        return input;
    }
    input.value = function(value) {
        input.el.value = value;
        return input;
    };
    input.min = function(min) {
        input.el.min = min;
        return input;
    };
    input.max = function(max) {
        input.el.max = max;
        return input;
    };
    return input;
}

function El(tag) {
    let self = this;

    self.el = document.createElement(tag);

    let children = [];
    let update = undefined;

    self.classes = function(classes) {
        self.el.className += classes;
        return self;
    };

    self.text = function(text) {
        self.el.innerText = text;
        return self;
    };

    self.children = function(c) {
        for (let i = 0; i < c.length; i++) {
            self.el.append(c[i].el);
            children.push(c[i]);
        }
        return self;
    };

    self.onclick = function(f) {
        self.el.onclick = f.bind(self);
        return self;
    };

    self.oninput = function(f) {
        self.el.oninput = f.bind(self);
        return self;
    }

    self.disabled = function(d) {
        self.el.disabled = d;
        return self;
    };

    self.style = function(s, v) {
        self.el.style[s] = v;
        return self;
    }

    self.onupdate = function(f) {
        update = f.bind(self);
        return self;
    };

    self.update = function(state) {
        if (update) {
            update(state);
        }

        for (let i = 0; i < children.length; i++) {
            children[i].update(state);
        }
    };
}
