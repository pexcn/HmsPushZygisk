import React from 'react';

type MaterialElement = React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement> & {
  [propName: string]: any;
};

declare global {
  namespace React {
    namespace JSX {
      interface IntrinsicElements {
        'md-list': MaterialElement;
        'md-list-item': MaterialElement;
        'md-switch': MaterialElement;
        'md-filled-text-field': MaterialElement;
        'md-circular-progress': MaterialElement;
        'md-icon': MaterialElement;
      }
    }
  }
}
