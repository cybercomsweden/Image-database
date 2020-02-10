module.exports = {
    "env": {
        "browser": true,
        "es6": true,
    },
    "extends": [
        "eslint:recommended",
        "plugin:react/recommended",
        "airbnb",
    ],
    "globals": {
        "Atomics": "readonly",
        "SharedArrayBuffer": "readonly",
    },
    "parserOptions": {
        "ecmaFeatures": {
            "jsx": true,
        },
        "ecmaVersion": 2018,
        "sourceType": "module",
    },
    "plugins": [
        "react",
    ],
    "rules": {
        // Override AirBnB's 2 space indentation
        "indent": [
            "error",
            4,
        ],
        "react/jsx-indent": ["error", 4],
        "react/jsx-indent-props": ["error", 4],

        "linebreak-style": [
            "error",
            "unix",
        ],
        "quotes": [
            "error",
            "double",
        ],
        "comma-dangle": [
            "error",
            "always-multiline",
        ],
        "react/prop-types": ["off"],
        "max-classes-per-file": ["off"],

        // Allows file extensions in import statements
        "import/extensions": ["off"],

        // Allow continue in loops
        "no-continue": ["off"],

        // Allow for (let x of y)
        "no-restricted-syntax": [
            "error",
            "ForInStatement",
            "LabeledStatement",
            "WithStatement",
        ],

        // Allow passing children as prop since React Router uses it
        "react/no-children-prop": ["off"],

        "import/prefer-default-export": ["off"],

        // Allow <Component {...attrs />
        "react/jsx-props-no-spreading": ["off"],
    },
    "settings": {
        "react": {
            "version": "detect",
        },
    },
}
